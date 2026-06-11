use ark_ff::PrimeField;
use multilinear::Multilinear;
use snark_lab_transcript::ProofTranscript;

use crate::round::DenseRoundPolynomial;

const DOMAIN: &[u8] = b"snark-lab/sumcheck-general/v1";

pub trait SumcheckPolynomial<F: PrimeField> {
    fn variables(&self) -> usize;

    fn max_individual_degree(&self) -> usize;

    fn bind_statement<T: ProofTranscript<F>>(&self, transcript: &mut T);

    fn round_polynomial(&self, prefix_challenges: &[F], round: usize) -> DenseRoundPolynomial<F>;

    fn evaluate_at(&self, point: &[F]) -> Result<F, GeneralVerifyError>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GeneralProof<F: PrimeField> {
    pub round_polynomials: Vec<DenseRoundPolynomial<F>>,
    pub final_evaluation: F,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GeneralVerifyError {
    WrongRoundCount,
    RoundDegreeTooHigh {
        round: usize,
        degree: usize,
        max_degree: usize,
    },
    RoundDoesNotMatchClaim {
        round: usize,
    },
    FinalEvaluationMismatch,
    InvalidEvaluationPoint,
}

fn bind_general_statement<F, T, P>(polynomial: &P, claimed_sum: F, transcript: &mut T)
where
    F: PrimeField,
    T: ProofTranscript<F>,
    P: SumcheckPolynomial<F>,
{
    transcript.append_domain_separator(DOMAIN);
    transcript.append_field_modulus();
    transcript.append_u64(b"num-variables", polynomial.variables() as u64);
    transcript.append_u64(
        b"max-individual-degree",
        polynomial.max_individual_degree() as u64,
    );
    transcript.append_field_element(b"claimed-sum", &claimed_sum);
    polynomial.bind_statement(transcript);
}

fn append_round<F: PrimeField, T: ProofTranscript<F>>(
    transcript: &mut T,
    round: usize,
    polynomial: &DenseRoundPolynomial<F>,
) {
    transcript.append_u64(b"round-index", round as u64);
    transcript.append_u64(b"round-degree", polynomial.degree() as u64);
    transcript.append_u64(b"round-coefficients", polynomial.coefficients.len() as u64);

    for coefficient in &polynomial.coefficients {
        transcript.append_field_element(b"round-coefficient", coefficient);
    }
}

pub fn prove_general<F, T, P>(
    polynomial: &P,
    claimed_sum: F,
    transcript: &mut T,
) -> Result<GeneralProof<F>, GeneralVerifyError>
where
    F: PrimeField,
    T: ProofTranscript<F>,
    P: SumcheckPolynomial<F>,
{
    bind_general_statement(polynomial, claimed_sum, transcript);

    let mut challenges = Vec::with_capacity(polynomial.variables());
    let mut round_polynomials = Vec::with_capacity(polynomial.variables());

    for round in 0..polynomial.variables() {
        let round_polynomial = polynomial.round_polynomial(&challenges, round);
        append_round(transcript, round, &round_polynomial);
        let challenge = transcript.challenge_scalar(b"round-challenge");

        challenges.push(challenge);
        round_polynomials.push(round_polynomial);
    }

    let final_evaluation = polynomial.evaluate_at(&challenges)?;

    Ok(GeneralProof {
        round_polynomials,
        final_evaluation,
    })
}

pub fn verify_general<F, T, P>(
    polynomial: &P,
    claimed_sum: F,
    proof: &GeneralProof<F>,
    transcript: &mut T,
) -> Result<Vec<F>, GeneralVerifyError>
where
    F: PrimeField,
    T: ProofTranscript<F>,
    P: SumcheckPolynomial<F>,
{
    if proof.round_polynomials.len() != polynomial.variables() {
        return Err(GeneralVerifyError::WrongRoundCount);
    }

    bind_general_statement(polynomial, claimed_sum, transcript);

    let mut claim = claimed_sum;
    let mut challenges = Vec::with_capacity(polynomial.variables());
    let max_degree = polynomial.max_individual_degree();

    for (round, round_polynomial) in proof.round_polynomials.iter().enumerate() {
        let degree = round_polynomial.degree();
        if degree > max_degree {
            return Err(GeneralVerifyError::RoundDegreeTooHigh {
                round,
                degree,
                max_degree,
            });
        }

        if round_polynomial.boolean_sum() != claim {
            return Err(GeneralVerifyError::RoundDoesNotMatchClaim { round });
        }

        append_round(transcript, round, round_polynomial);
        let challenge = transcript.challenge_scalar(b"round-challenge");

        claim = round_polynomial.evaluate(challenge);
        challenges.push(challenge);
    }

    let oracle_evaluation = polynomial.evaluate_at(&challenges)?;

    if claim != proof.final_evaluation || oracle_evaluation != proof.final_evaluation {
        return Err(GeneralVerifyError::FinalEvaluationMismatch);
    }

    Ok(challenges)
}

impl<F: PrimeField> SumcheckPolynomial<F> for Multilinear<F> {
    fn variables(&self) -> usize {
        self.variables()
    }

    fn max_individual_degree(&self) -> usize {
        1
    }

    fn bind_statement<T: ProofTranscript<F>>(&self, transcript: &mut T) {
        transcript.append_u64(b"oracle-length", self.evaluations().len() as u64);
        for evaluation in self.evaluations() {
            transcript.append_field_element(b"oracle-evaluation", evaluation);
        }
    }

    fn round_polynomial(&self, prefix_challenges: &[F], _round: usize) -> DenseRoundPolynomial<F> {
        let mut folded = self.clone();

        for challenge in prefix_challenges {
            folded = folded.fold_first(*challenge);
        }

        let evaluation_at_zero = folded
            .evaluations()
            .chunks_exact(2)
            .map(|pair| pair[0])
            .sum();

        let evaluation_at_one = folded
            .evaluations()
            .chunks_exact(2)
            .map(|pair| pair[1])
            .sum();

        DenseRoundPolynomial::linear(evaluation_at_zero, evaluation_at_one)
    }

    fn evaluate_at(&self, point: &[F]) -> Result<F, GeneralVerifyError> {
        self.evaluate(point)
            .map_err(|_| GeneralVerifyError::InvalidEvaluationPoint)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ark_bls12_381::Fr;
    use ark_ff::Field;
    use snark_lab_transcript::MerlinTranscript;

    fn fixture_polynomial(offset: u64) -> Multilinear<Fr> {
        Multilinear::new((0u64..8).map(|value| Fr::from(value + offset)).collect()).unwrap()
    }

    fn prove_fixture(polynomial: &Multilinear<Fr>) -> (Fr, GeneralProof<Fr>) {
        let claim = polynomial.sum_hypercube();
        let mut transcript = MerlinTranscript::new(b"general-sumcheck-test");
        let proof = prove_general(polynomial, claim, &mut transcript).unwrap();
        (claim, proof)
    }

    #[test]
    fn general_sumcheck_accepts_multilinear_table() {
        let polynomial = fixture_polynomial(0);
        let (claim, proof) = prove_fixture(&polynomial);

        let mut transcript = MerlinTranscript::new(b"general-sumcheck-test");
        assert!(verify_general(&polynomial, claim, &proof, &mut transcript).is_ok());
    }

    #[test]
    fn general_sumcheck_rejects_wrong_claim() {
        let polynomial = fixture_polynomial(0);
        let (claim, proof) = prove_fixture(&polynomial);

        let mut transcript = MerlinTranscript::new(b"general-sumcheck-test");
        assert_eq!(
            verify_general(&polynomial, claim + Fr::ONE, &proof, &mut transcript),
            Err(GeneralVerifyError::RoundDoesNotMatchClaim { round: 0 })
        );
    }

    #[test]
    fn general_sumcheck_rejects_high_degree_round_message() {
        let polynomial = fixture_polynomial(0);
        let (claim, mut proof) = prove_fixture(&polynomial);

        proof.round_polynomials[0].coefficients.push(Fr::ONE);

        let mut transcript = MerlinTranscript::new(b"general-sumcheck-test");
        assert_eq!(
            verify_general(&polynomial, claim, &proof, &mut transcript),
            Err(GeneralVerifyError::RoundDegreeTooHigh {
                round: 0,
                degree: 2,
                max_degree: 1
            })
        );
    }

    #[test]
    fn general_sumcheck_rejects_tampered_final_evaluation() {
        let polynomial = fixture_polynomial(0);
        let (claim, mut proof) = prove_fixture(&polynomial);

        proof.final_evaluation += Fr::ONE;

        let mut transcript = MerlinTranscript::new(b"general-sumcheck-test");
        assert_eq!(
            verify_general(&polynomial, claim, &proof, &mut transcript),
            Err(GeneralVerifyError::FinalEvaluationMismatch)
        );
    }

    #[test]
    fn changing_statement_changes_general_challenges() {
        let polynomial = fixture_polynomial(0);
        let (claim, proof) = prove_fixture(&polynomial);

        let mut left = MerlinTranscript::new(b"general-sumcheck-test");
        let original = verify_general(&polynomial, claim, &proof, &mut left).unwrap();

        let changed_polynomial = fixture_polynomial(1);
        let mut right = MerlinTranscript::new(b"general-sumcheck-test");
        let changed = verify_general(&changed_polynomial, claim, &proof, &mut right);

        assert!(changed.is_err());

        let mut direct_left = MerlinTranscript::new(b"general-sumcheck-test");
        let proof_for_original = prove_general(&polynomial, claim, &mut direct_left).unwrap();

        let changed_claim = changed_polynomial.sum_hypercube();
        let mut direct_right = MerlinTranscript::new(b"general-sumcheck-test");
        let proof_for_changed =
            prove_general(&changed_polynomial, changed_claim, &mut direct_right).unwrap();

        let mut verify_left = MerlinTranscript::new(b"general-sumcheck-test");
        let original_challenges =
            verify_general(&polynomial, claim, &proof_for_original, &mut verify_left).unwrap();

        let mut verify_right = MerlinTranscript::new(b"general-sumcheck-test");
        let changed_challenges = verify_general(
            &changed_polynomial,
            changed_claim,
            &proof_for_changed,
            &mut verify_right,
        )
        .unwrap();

        assert_ne!(original, changed_challenges);
        assert_ne!(original_challenges, changed_challenges);
    }
}
