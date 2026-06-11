use ark_ff::PrimeField;
use multilinear::Multilinear;
use snark_lab_transcript::ProofTranscript;

/// Shape-level validation errors shared by polynomial-commitment backends.
///
/// These are protocol-facing errors, not backend-specific cryptographic failures.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PcsShapeError {
    UnsupportedVariableCount { requested: usize, supported: usize },
    PointLengthMismatch { expected: usize, actual: usize },
}

/// Validate that a backend key supports the requested number of variables.
pub fn validate_supported_variables(
    supported: usize,
    requested: usize,
) -> Result<(), PcsShapeError> {
    if requested > supported {
        return Err(PcsShapeError::UnsupportedVariableCount {
            requested,
            supported,
        });
    }

    Ok(())
}

/// Validate that an opening point has one coordinate per multilinear variable.
pub fn validate_opening_point(expected: usize, actual: usize) -> Result<(), PcsShapeError> {
    if expected != actual {
        return Err(PcsShapeError::PointLengthMismatch { expected, actual });
    }

    Ok(())
}

/// Production-facing multilinear polynomial commitment interface.
///
/// This trait is intentionally separate from `MultilinearOracle`.
///
/// `MultilinearOracle` is the protocol-side abstraction used by Sumcheck.
/// `MultilinearPcs` is the backend-side abstraction needed for real KZG, IPA,
/// or FRI implementations.
///
/// A backend implementing this trait must provide:
///
/// - public parameter generation,
/// - prover/verifier key separation,
/// - commitment to a multilinear polynomial,
/// - opening at an arbitrary verifier challenge point,
/// - verification of that opening against the verifier key and commitment,
/// - transcript binding for Fiat-Shamir soundness.
pub trait MultilinearPcs<F: PrimeField> {
    type PublicParameters: Clone + core::fmt::Debug;
    type ProverKey: Clone + core::fmt::Debug;
    type VerifierKey: Clone + core::fmt::Debug;
    type Commitment: Clone + core::fmt::Debug + PartialEq + Eq;
    type Opening: Clone + core::fmt::Debug + PartialEq + Eq;
    type Error: Clone + core::fmt::Debug + PartialEq + Eq;

    /// Create public parameters supporting up to `max_variables` multilinear variables.
    ///
    /// Real backends may use structured reference strings, transparent setup, or
    /// trusted setup depending on the commitment scheme.
    fn setup(max_variables: usize) -> Result<Self::PublicParameters, Self::Error>;

    /// Split public parameters into prover and verifier keys.
    fn trim(
        public_parameters: &Self::PublicParameters,
        supported_variables: usize,
    ) -> Result<(Self::ProverKey, Self::VerifierKey), Self::Error>;

    /// Commit to the full multilinear evaluation table.
    fn commit(
        prover_key: &Self::ProverKey,
        polynomial: &Multilinear<F>,
    ) -> Result<Self::Commitment, Self::Error>;

    /// Bind the commitment into the Fiat-Shamir transcript.
    fn bind_commitment<T: ProofTranscript<F>>(commitment: &Self::Commitment, transcript: &mut T);

    /// Open the committed multilinear polynomial at `point`.
    fn open(
        prover_key: &Self::ProverKey,
        polynomial: &Multilinear<F>,
        point: &[F],
    ) -> Result<Self::Opening, Self::Error>;

    /// Verify an opening and return the opened field value.
    fn verify(
        verifier_key: &Self::VerifierKey,
        commitment: &Self::Commitment,
        point: &[F],
        opening: &Self::Opening,
    ) -> Result<F, Self::Error>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn supported_variable_validation_accepts_within_bound() {
        assert_eq!(validate_supported_variables(8, 8), Ok(()));
        assert_eq!(validate_supported_variables(8, 0), Ok(()));
    }

    #[test]
    fn supported_variable_validation_rejects_over_bound() {
        assert_eq!(
            validate_supported_variables(8, 9),
            Err(PcsShapeError::UnsupportedVariableCount {
                requested: 9,
                supported: 8
            })
        );
    }

    #[test]
    fn opening_point_validation_accepts_exact_length() {
        assert_eq!(validate_opening_point(3, 3), Ok(()));
    }

    #[test]
    fn opening_point_validation_rejects_wrong_length() {
        assert_eq!(
            validate_opening_point(3, 2),
            Err(PcsShapeError::PointLengthMismatch {
                expected: 3,
                actual: 2
            })
        );
    }
}
