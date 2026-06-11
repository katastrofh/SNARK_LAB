#![forbid(unsafe_code)]

//! Fiat–Shamir Zerocheck reduced to equality-weighted Sumcheck.

use ark_ff::PrimeField;
use multilinear::{eq_evaluations, Multilinear};
use snark_lab_transcript::ProofTranscript;
use sumcheck::{Proof as SumcheckProof, VerifyError as SumcheckError};

const DOMAIN: &[u8] = b"snark-lab/zerocheck/v1";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Proof<F: PrimeField> {
    pub sumcheck: SumcheckProof<F>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VerifyError {
    Sumcheck(SumcheckError),
}

impl core::fmt::Display for VerifyError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Sumcheck(error) => write!(formatter, "Sumcheck verification failed: {error}"),
        }
    }
}

impl std::error::Error for VerifyError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Sumcheck(error) => Some(error),
        }
    }
}

pub fn weighted_polynomial<F: PrimeField>(
    constraints: &Multilinear<F>,
    mixing_point: &[F],
) -> Result<Multilinear<F>, multilinear::Error> {
    if mixing_point.len() != constraints.variables() {
        return Err(multilinear::Error::PointDimensionMismatch);
    }
    Multilinear::new(
        constraints
            .evaluations()
            .iter()
            .copied()
            .zip(eq_evaluations(mixing_point))
            .map(|(constraint, equality)| constraint * equality)
            .collect(),
    )
}

fn derive_mixing_point<F: PrimeField, T: ProofTranscript<F>>(
    constraints: &Multilinear<F>,
    transcript: &mut T,
) -> Vec<F> {
    transcript.append_domain_separator(DOMAIN);
    transcript.append_field_modulus();
    transcript.append_u64(b"num-variables", constraints.variables() as u64);
    transcript.append_u64(b"constraint-length", constraints.evaluations().len() as u64);
    // This is a transparent binding. A commitment-backed oracle will replace it.
    for evaluation in constraints.evaluations() {
        transcript.append_field_element(b"constraint-evaluation", evaluation);
    }
    transcript.challenge_vector(b"mixing-coordinate", constraints.variables())
}

pub fn prove<F: PrimeField, T: ProofTranscript<F>>(
    constraints: &Multilinear<F>,
    transcript: &mut T,
) -> Proof<F> {
    let mixing_point = derive_mixing_point(constraints, transcript);
    let weighted = weighted_polynomial(constraints, &mixing_point)
        .expect("Fiat-Shamir mixing point has the constraint dimension");
    Proof {
        sumcheck: sumcheck::prove(&weighted, F::ZERO, transcript),
    }
}

pub fn verify<F: PrimeField, T: ProofTranscript<F>>(
    constraints: &Multilinear<F>,
    proof: &Proof<F>,
    transcript: &mut T,
) -> Result<Vec<F>, VerifyError> {
    let mixing_point = derive_mixing_point(constraints, transcript);
    let weighted = weighted_polynomial(constraints, &mixing_point)
        .expect("Fiat-Shamir mixing point has the constraint dimension");
    sumcheck::verify(&weighted, F::ZERO, &proof.sumcheck, transcript)
        .map_err(VerifyError::Sumcheck)?;
    Ok(mixing_point)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ark_bls12_381::Fr;
    use ark_ff::{AdditiveGroup, Field};
    use snark_lab_transcript::MerlinTranscript;

    #[test]
    fn zero_table_verifies() {
        let constraints = Multilinear::new(vec![Fr::ZERO; 8]).unwrap();
        let mut prover = MerlinTranscript::new(b"zerocheck-test");
        let proof = prove(&constraints, &mut prover);
        let mut verifier = MerlinTranscript::new(b"zerocheck-test");
        assert!(verify(&constraints, &proof, &mut verifier).is_ok());
    }

    #[test]
    fn nonzero_table_is_rejected() {
        let constraints =
            Multilinear::new([0_u64, 0, 9, 0].into_iter().map(Fr::from).collect()).unwrap();
        let mut prover = MerlinTranscript::new(b"zerocheck-test");
        let proof = prove(&constraints, &mut prover);
        let mut verifier = MerlinTranscript::new(b"zerocheck-test");
        assert!(verify(&constraints, &proof, &mut verifier).is_err());
    }

    #[test]
    fn constraint_oracle_is_bound_before_mixing_challenge() {
        let zero = Multilinear::new(vec![Fr::ZERO; 4]).unwrap();
        let changed = Multilinear::new(vec![Fr::ZERO, Fr::ONE, Fr::ZERO, Fr::ZERO]).unwrap();
        let mut left = MerlinTranscript::new(b"zerocheck-test");
        let mut right = MerlinTranscript::new(b"zerocheck-test");
        assert_ne!(
            derive_mixing_point(&zero, &mut left),
            derive_mixing_point(&changed, &mut right)
        );
    }
}
