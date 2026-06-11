//! Fiat–Shamir Sumcheck for transparent multilinear evaluation tables.
//!
//! Prover messages are bound to a Merlin transcript before each challenge is
//! derived. The transparent oracle is deliberately separated from a future
//! polynomial-commitment backend.

pub mod general;
pub mod round;

pub use general::{
    prove_general, verify_general, GeneralProof, GeneralVerifyError, SumcheckPolynomial,
};
pub use round::DenseRoundPolynomial;

use ark_ff::PrimeField;
use multilinear::Multilinear;
use snark_lab_transcript::ProofTranscript;

const DOMAIN: &[u8] = b"snark-lab/sumcheck/v1";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RoundPolynomial<F: PrimeField> {
    pub evaluation_at_zero: F,
    pub evaluation_at_one: F,
}

impl<F: PrimeField> RoundPolynomial<F> {
    pub fn evaluate(&self, point: F) -> F {
        self.evaluation_at_zero + point * (self.evaluation_at_one - self.evaluation_at_zero)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Proof<F: PrimeField> {
    pub round_polynomials: Vec<RoundPolynomial<F>>,
    pub final_evaluation: F,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VerifyError {
    WrongRoundCount,
    RoundDoesNotMatchClaim { round: usize },
    FinalEvaluationMismatch,
}

fn bind_statement<F: PrimeField, T: ProofTranscript<F>>(
    transcript: &mut T,
    polynomial: &Multilinear<F>,
    claimed_sum: F,
) {
    transcript.append_domain_separator(DOMAIN);
    transcript.append_field_modulus();
    transcript.append_u64(b"num-variables", polynomial.variables() as u64);
    transcript.append_field_element(b"claimed-sum", &claimed_sum);
    transcript.append_u64(b"oracle-length", polynomial.evaluations().len() as u64);
    for evaluation in polynomial.evaluations() {
        transcript.append_field_element(b"oracle-evaluation", evaluation);
    }
}

fn append_round<F: PrimeField, T: ProofTranscript<F>>(
    transcript: &mut T,
    round: usize,
    polynomial: &RoundPolynomial<F>,
) {
    transcript.append_u64(b"round-index", round as u64);
    transcript.append_field_element(b"round-evaluation-0", &polynomial.evaluation_at_zero);
    transcript.append_field_element(b"round-evaluation-1", &polynomial.evaluation_at_one);
}

pub fn prove<F: PrimeField, T: ProofTranscript<F>>(
    polynomial: &Multilinear<F>,
    claimed_sum: F,
    transcript: &mut T,
) -> Proof<F> {
    bind_statement(transcript, polynomial, claimed_sum);
    let mut folded = polynomial.clone();
    let mut round_polynomials = Vec::with_capacity(polynomial.variables());

    for round in 0..polynomial.variables() {
        let message = RoundPolynomial {
            evaluation_at_zero: folded
                .evaluations()
                .chunks_exact(2)
                .map(|pair| pair[0])
                .sum(),
            evaluation_at_one: folded
                .evaluations()
                .chunks_exact(2)
                .map(|pair| pair[1])
                .sum(),
        };
        append_round(transcript, round, &message);
        let challenge = transcript.challenge_scalar(b"round-challenge");
        folded = folded.fold_first(challenge);
        round_polynomials.push(message);
    }

    Proof {
        round_polynomials,
        final_evaluation: folded.evaluations()[0],
    }
}

/// Derives the verifier challenges from the statement and proof messages.
pub fn derive_challenges<F: PrimeField, T: ProofTranscript<F>>(
    polynomial: &Multilinear<F>,
    claimed_sum: F,
    proof: &Proof<F>,
    transcript: &mut T,
) -> Vec<F> {
    bind_statement(transcript, polynomial, claimed_sum);
    proof
        .round_polynomials
        .iter()
        .enumerate()
        .map(|(round, message)| {
            append_round(transcript, round, message);
            transcript.challenge_scalar(b"round-challenge")
        })
        .collect()
}

pub fn verify<F: PrimeField, T: ProofTranscript<F>>(
    polynomial: &Multilinear<F>,
    claimed_sum: F,
    proof: &Proof<F>,
    transcript: &mut T,
) -> Result<Vec<F>, VerifyError> {
    if proof.round_polynomials.len() != polynomial.variables() {
        return Err(VerifyError::WrongRoundCount);
    }

    let challenges = derive_challenges(polynomial, claimed_sum, proof, transcript);
    let mut claim = claimed_sum;
    for (round, (message, challenge)) in proof
        .round_polynomials
        .iter()
        .zip(challenges.iter().copied())
        .enumerate()
    {
        if message.evaluation_at_zero + message.evaluation_at_one != claim {
            return Err(VerifyError::RoundDoesNotMatchClaim { round });
        }
        claim = message.evaluate(challenge);
    }

    if claim != proof.final_evaluation
        || polynomial.evaluate(&challenges).ok() != Some(proof.final_evaluation)
    {
        return Err(VerifyError::FinalEvaluationMismatch);
    }
    Ok(challenges)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ark_bls12_381::Fr;
    use ark_ff::Field;
    use snark_lab_transcript::MerlinTranscript;

    fn fixture_polynomial(offset: u64) -> Multilinear<Fr> {
        Multilinear::new((0..8).map(|value| Fr::from(value + offset)).collect()).unwrap()
    }

    fn prove_fixture(polynomial: &Multilinear<Fr>) -> (Fr, Proof<Fr>) {
        let claim = polynomial.sum_hypercube();
        let mut transcript = MerlinTranscript::new(b"sumcheck-test");
        (claim, prove(polynomial, claim, &mut transcript))
    }

    #[test]
    fn valid_sumcheck_accepts() {
        let polynomial = fixture_polynomial(0);
        let (claim, proof) = prove_fixture(&polynomial);
        let mut transcript = MerlinTranscript::new(b"sumcheck-test");
        assert!(verify(&polynomial, claim, &proof, &mut transcript).is_ok());
    }

    #[test]
    fn tampered_claim_is_rejected() {
        let polynomial = fixture_polynomial(0);
        let (claim, proof) = prove_fixture(&polynomial);
        let mut transcript = MerlinTranscript::new(b"sumcheck-test");
        assert!(verify(&polynomial, claim + Fr::ONE, &proof, &mut transcript).is_err());
    }

    #[test]
    fn tampering_is_rejected() {
        let polynomial = fixture_polynomial(0);
        let (claim, mut proof) = prove_fixture(&polynomial);
        proof.round_polynomials[1].evaluation_at_one += Fr::ONE;
        let mut transcript = MerlinTranscript::new(b"sumcheck-test");
        assert!(verify(&polynomial, claim, &proof, &mut transcript).is_err());

        let (_, mut proof) = prove_fixture(&polynomial);
        proof.final_evaluation += Fr::ONE;
        let mut transcript = MerlinTranscript::new(b"sumcheck-test");
        assert!(verify(&polynomial, claim, &proof, &mut transcript).is_err());
    }

    #[test]
    fn same_transcript_same_challenges() {
        let polynomial = fixture_polynomial(0);
        let (claim, proof) = prove_fixture(&polynomial);
        let mut left = MerlinTranscript::new(b"sumcheck-test");
        let mut right = MerlinTranscript::new(b"sumcheck-test");
        assert_eq!(
            derive_challenges(&polynomial, claim, &proof, &mut left),
            derive_challenges(&polynomial, claim, &proof, &mut right)
        );
    }

    #[test]
    fn public_input_and_round_messages_bind_challenges() {
        let polynomial = fixture_polynomial(0);
        let (claim, proof) = prove_fixture(&polynomial);
        let mut transcript = MerlinTranscript::new(b"sumcheck-test");
        let original = derive_challenges(&polynomial, claim, &proof, &mut transcript);

        let changed_polynomial = fixture_polynomial(1);
        let mut transcript = MerlinTranscript::new(b"sumcheck-test");
        let changed_statement =
            derive_challenges(&changed_polynomial, claim, &proof, &mut transcript);
        assert_ne!(original[0], changed_statement[0]);

        let mut changed_proof = proof.clone();
        changed_proof.round_polynomials[0].evaluation_at_zero += Fr::ONE;
        let mut transcript = MerlinTranscript::new(b"sumcheck-test");
        let changed_message =
            derive_challenges(&polynomial, claim, &changed_proof, &mut transcript);
        assert_ne!(original[0], changed_message[0]);
        assert_ne!(original[1], changed_message[1]);
    }
}
