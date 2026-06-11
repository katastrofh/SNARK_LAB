#![forbid(unsafe_code)]
//! Oracle abstraction for commitment-backed protocol plumbing.
//!
//! The first backend is transparent and non-succinct. The point of this crate
//! is to separate protocol logic from the concrete oracle representation so a
//! future KZG/IPA/FRI backend can replace full-table verification.

use ark_ff::PrimeField;
use multilinear::Multilinear;
use snark_lab_transcript::ProofTranscript;

pub mod ipa;
pub mod ipa_transcript;
pub use ipa::{
    IpaBackend, IpaBackendError, IpaCommitment, IpaOpening, IpaProverKey, IpaPublicParameters,
    IpaVerifierKey,
};
pub use ipa_transcript::{
    absorb_ipa_reduction_round, bind_ipa_opening_statement, expected_ipa_rounds,
    validate_ipa_round_count, IpaRoundSide, IpaTranscriptError, IpaTranscriptRound,
};
pub mod pcs;
pub use pcs::{
    validate_opening_point, validate_supported_variables, MultilinearPcs, PcsShapeError,
};

/// Commitment/opening interface for multilinear evaluation oracles.
pub trait MultilinearOracle<F: PrimeField> {
    type Commitment: Clone + core::fmt::Debug + PartialEq + Eq;
    type Opening: Clone + core::fmt::Debug + PartialEq + Eq;
    type Error: Clone + core::fmt::Debug + PartialEq + Eq;

    fn variables(&self) -> usize;

    fn commit(&self) -> Self::Commitment;

    fn bind_commitment<T: ProofTranscript<F>>(commitment: &Self::Commitment, transcript: &mut T);

    fn open(&self, point: &[F]) -> Result<Self::Opening, Self::Error>;

    fn verify_opening(
        commitment: &Self::Commitment,
        point: &[F],
        opening: &Self::Opening,
    ) -> Result<F, Self::Error>;
}

/// Transparent commitment carrying the full evaluation table.
///
/// This is intentionally not succinct. It preserves current behavior while
/// creating the abstraction boundary needed for commitment-backed Sumcheck.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TransparentCommitment<F: PrimeField> {
    pub variables: usize,
    pub evaluations: Vec<F>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TransparentOpening<F: PrimeField> {
    pub value: F,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TransparentOracleError {
    PointLengthMismatch { expected: usize, actual: usize },
    InvalidTable,
    InvalidOpening,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TransparentOracle<F: PrimeField> {
    polynomial: Multilinear<F>,
}

impl<F: PrimeField> TransparentOracle<F> {
    pub fn new(polynomial: Multilinear<F>) -> Self {
        Self { polynomial }
    }

    pub fn polynomial(&self) -> &Multilinear<F> {
        &self.polynomial
    }

    pub fn sum_hypercube(&self) -> F {
        self.polynomial.sum_hypercube()
    }
}

impl<F: PrimeField> From<Multilinear<F>> for TransparentOracle<F> {
    fn from(polynomial: Multilinear<F>) -> Self {
        Self::new(polynomial)
    }
}

impl<F: PrimeField> MultilinearOracle<F> for TransparentOracle<F> {
    type Commitment = TransparentCommitment<F>;
    type Opening = TransparentOpening<F>;
    type Error = TransparentOracleError;

    fn variables(&self) -> usize {
        self.polynomial.variables()
    }

    fn commit(&self) -> Self::Commitment {
        TransparentCommitment {
            variables: self.polynomial.variables(),
            evaluations: self.polynomial.evaluations().to_vec(),
        }
    }

    fn bind_commitment<T: ProofTranscript<F>>(commitment: &Self::Commitment, transcript: &mut T) {
        transcript.append_u64(b"oracle-variables", commitment.variables as u64);
        transcript.append_u64(b"oracle-length", commitment.evaluations.len() as u64);
        for evaluation in &commitment.evaluations {
            transcript.append_field_element(b"oracle-evaluation", evaluation);
        }
    }

    fn open(&self, point: &[F]) -> Result<Self::Opening, Self::Error> {
        if point.len() != self.polynomial.variables() {
            return Err(TransparentOracleError::PointLengthMismatch {
                expected: self.polynomial.variables(),
                actual: point.len(),
            });
        }

        Ok(TransparentOpening {
            value: self
                .polynomial
                .evaluate(point)
                .map_err(|_| TransparentOracleError::InvalidTable)?,
        })
    }

    fn verify_opening(
        commitment: &Self::Commitment,
        point: &[F],
        opening: &Self::Opening,
    ) -> Result<F, Self::Error> {
        if point.len() != commitment.variables {
            return Err(TransparentOracleError::PointLengthMismatch {
                expected: commitment.variables,
                actual: point.len(),
            });
        }

        let polynomial = Multilinear::new(commitment.evaluations.clone())
            .map_err(|_| TransparentOracleError::InvalidTable)?;

        if polynomial.variables() != commitment.variables {
            return Err(TransparentOracleError::InvalidTable);
        }

        let expected = polynomial
            .evaluate(point)
            .map_err(|_| TransparentOracleError::InvalidTable)?;

        if expected != opening.value {
            return Err(TransparentOracleError::InvalidOpening);
        }

        Ok(opening.value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ark_bls12_381::Fr;

    #[test]
    fn transparent_oracle_opens_and_verifies() {
        let polynomial = Multilinear::new((0u64..8).map(Fr::from).collect()).expect("valid table");
        let oracle = TransparentOracle::new(polynomial);
        let commitment = oracle.commit();
        let point = [Fr::from(3u64), Fr::from(5u64), Fr::from(7u64)];
        let opening = oracle.open(&point).expect("opening");

        let verified =
            TransparentOracle::verify_opening(&commitment, &point, &opening).expect("verified");

        assert_eq!(verified, opening.value);
    }

    #[test]
    fn transparent_oracle_rejects_tampered_opening() {
        let polynomial = Multilinear::new((0u64..8).map(Fr::from).collect()).expect("valid table");
        let oracle = TransparentOracle::new(polynomial);
        let commitment = oracle.commit();
        let point = [Fr::from(3u64), Fr::from(5u64), Fr::from(7u64)];
        let mut opening = oracle.open(&point).expect("opening");
        opening.value += Fr::from(1u64);

        assert!(TransparentOracle::verify_opening(&commitment, &point, &opening).is_err());
    }

    #[test]
    fn transparent_oracle_rejects_wrong_point_length() {
        let polynomial = Multilinear::new((0u64..8).map(Fr::from).collect()).expect("valid table");
        let oracle = TransparentOracle::new(polynomial);
        let point = [Fr::from(3u64), Fr::from(5u64)];

        assert!(oracle.open(&point).is_err());
    }
}
