use ark_ff::PrimeField;
use snark_lab_oracle::{MultilinearOracle, TransparentOracle, TransparentOracleError};
use snark_lab_transcript::ProofTranscript;

use crate::{Proof, RoundPolynomial};

const DOMAIN: &[u8] = b"snark-lab/sumcheck-oracle/v1";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OracleProof<F, O>
where
    F: PrimeField,
    O: MultilinearOracle<F>,
{
    pub commitment: O::Commitment,
    pub sumcheck_proof: Proof<F>,
    pub final_opening: O::Opening,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OracleVerifyError<E> {
    WrongRoundCount,
    RoundDoesNotMatchClaim { round: usize },
    Opening(E),
    FinalEvaluationMismatch,
}

fn bind_oracle_statement<F, T, O>(
    transcript: &mut T,
    variables: usize,
    claimed_sum: F,
    commitment: &O::Commitment,
) where
    F: PrimeField,
    T: ProofTranscript<F>,
    O: MultilinearOracle<F>,
{
    transcript.append_domain_separator(DOMAIN);
    transcript.append_field_modulus();
    transcript.append_u64(b"num-variables", variables as u64);
    transcript.append_field_element(b"claimed-sum", &claimed_sum);
    O::bind_commitment(commitment, transcript);
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

pub fn prove_with_transparent_oracle<F, T>(
    oracle: &TransparentOracle<F>,
    claimed_sum: F,
    transcript: &mut T,
) -> Result<OracleProof<F, TransparentOracle<F>>, TransparentOracleError>
where
    F: PrimeField,
    T: ProofTranscript<F>,
{
    let commitment = oracle.commit();

    bind_oracle_statement::<F, T, TransparentOracle<F>>(
        transcript,
        oracle.variables(),
        claimed_sum,
        &commitment,
    );

    let mut folded = oracle.polynomial().clone();
    let mut round_polynomials = Vec::with_capacity(oracle.variables());
    let mut challenges = Vec::with_capacity(oracle.variables());

    for round in 0..oracle.variables() {
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
        challenges.push(challenge);
        round_polynomials.push(message);
    }

    let sumcheck_proof = Proof {
        round_polynomials,
        final_evaluation: folded.evaluations()[0],
    };

    let final_opening = oracle.open(&challenges)?;

    Ok(OracleProof {
        commitment,
        sumcheck_proof,
        final_opening,
    })
}

pub fn verify_with_oracle<F, T, O>(
    commitment: &O::Commitment,
    variables: usize,
    claimed_sum: F,
    proof: &Proof<F>,
    final_opening: &O::Opening,
    transcript: &mut T,
) -> Result<Vec<F>, OracleVerifyError<O::Error>>
where
    F: PrimeField,
    T: ProofTranscript<F>,
    O: MultilinearOracle<F>,
{
    if proof.round_polynomials.len() != variables {
        return Err(OracleVerifyError::WrongRoundCount);
    }

    bind_oracle_statement::<F, T, O>(transcript, variables, claimed_sum, commitment);

    let mut claim = claimed_sum;
    let mut challenges = Vec::with_capacity(variables);

    for (round, message) in proof.round_polynomials.iter().enumerate() {
        if message.evaluation_at_zero + message.evaluation_at_one != claim {
            return Err(OracleVerifyError::RoundDoesNotMatchClaim { round });
        }

        append_round(transcript, round, message);
        let challenge = transcript.challenge_scalar(b"round-challenge");

        claim = message.evaluate(challenge);
        challenges.push(challenge);
    }

    let opened_value = O::verify_opening(commitment, &challenges, final_opening)
        .map_err(OracleVerifyError::Opening)?;

    if claim != proof.final_evaluation || opened_value != proof.final_evaluation {
        return Err(OracleVerifyError::FinalEvaluationMismatch);
    }

    Ok(challenges)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ark_bls12_381::Fr;
    use ark_ff::Field;
    use multilinear::Multilinear;
    use snark_lab_transcript::MerlinTranscript;

    fn fixture_oracle(offset: u64) -> TransparentOracle<Fr> {
        let polynomial =
            Multilinear::new((0u64..8).map(|value| Fr::from(value + offset)).collect()).unwrap();
        TransparentOracle::new(polynomial)
    }

    fn prove_fixture(
        oracle: &TransparentOracle<Fr>,
    ) -> (Fr, OracleProof<Fr, TransparentOracle<Fr>>) {
        let claim = oracle.sum_hypercube();
        let mut transcript = MerlinTranscript::new(b"oracle-sumcheck-test");
        let proof = prove_with_transparent_oracle(oracle, claim, &mut transcript).unwrap();
        (claim, proof)
    }

    #[test]
    fn oracle_sumcheck_accepts_transparent_opening() {
        let oracle = fixture_oracle(0);
        let (claim, proof) = prove_fixture(&oracle);

        let mut transcript = MerlinTranscript::new(b"oracle-sumcheck-test");
        let challenges = verify_with_oracle::<Fr, _, TransparentOracle<Fr>>(
            &proof.commitment,
            oracle.variables(),
            claim,
            &proof.sumcheck_proof,
            &proof.final_opening,
            &mut transcript,
        )
        .unwrap();

        assert_eq!(challenges.len(), oracle.variables());
    }

    #[test]
    fn oracle_sumcheck_rejects_tampered_opening() {
        let oracle = fixture_oracle(0);
        let (claim, mut proof) = prove_fixture(&oracle);

        proof.final_opening.value += Fr::ONE;

        let mut transcript = MerlinTranscript::new(b"oracle-sumcheck-test");
        assert_eq!(
            verify_with_oracle::<Fr, _, TransparentOracle<Fr>>(
                &proof.commitment,
                oracle.variables(),
                claim,
                &proof.sumcheck_proof,
                &proof.final_opening,
                &mut transcript,
            ),
            Err(OracleVerifyError::Opening(
                TransparentOracleError::InvalidOpening
            ))
        );
    }

    #[test]
    fn oracle_sumcheck_rejects_changed_commitment() {
        let oracle = fixture_oracle(0);
        let (claim, mut proof) = prove_fixture(&oracle);

        proof.commitment.evaluations[0] += Fr::ONE;

        let mut transcript = MerlinTranscript::new(b"oracle-sumcheck-test");
        assert!(verify_with_oracle::<Fr, _, TransparentOracle<Fr>>(
            &proof.commitment,
            oracle.variables(),
            claim,
            &proof.sumcheck_proof,
            &proof.final_opening,
            &mut transcript,
        )
        .is_err());
    }

    #[test]
    fn oracle_sumcheck_rejects_tampered_round_message() {
        let oracle = fixture_oracle(0);
        let (claim, mut proof) = prove_fixture(&oracle);

        proof.sumcheck_proof.round_polynomials[0].evaluation_at_one += Fr::ONE;

        let mut transcript = MerlinTranscript::new(b"oracle-sumcheck-test");
        assert_eq!(
            verify_with_oracle::<Fr, _, TransparentOracle<Fr>>(
                &proof.commitment,
                oracle.variables(),
                claim,
                &proof.sumcheck_proof,
                &proof.final_opening,
                &mut transcript,
            ),
            Err(OracleVerifyError::RoundDoesNotMatchClaim { round: 0 })
        );
    }

    #[test]
    fn oracle_sumcheck_rejects_wrong_round_count() {
        let oracle = fixture_oracle(0);
        let (claim, mut proof) = prove_fixture(&oracle);

        proof.sumcheck_proof.round_polynomials.pop();

        let mut transcript = MerlinTranscript::new(b"oracle-sumcheck-test");
        assert_eq!(
            verify_with_oracle::<Fr, _, TransparentOracle<Fr>>(
                &proof.commitment,
                oracle.variables(),
                claim,
                &proof.sumcheck_proof,
                &proof.final_opening,
                &mut transcript,
            ),
            Err(OracleVerifyError::WrongRoundCount)
        );
    }
}
