use ark_ff::PrimeField;
use multilinear::Multilinear;
use snark_lab_transcript::ProofTranscript;

use crate::pcs::{
    validate_opening_point, validate_supported_variables, MultilinearPcs, PcsShapeError,
};

const IPA_TRANSCRIPT_DOMAIN: &[u8] = b"snark-lab/ipa-pcs/v1";

/// Public parameters for an IPA-style multilinear polynomial commitment backend.
///
/// This type is intentionally shape-only for now. It establishes the production
/// API boundary without pretending that an IPA commitment scheme has already
/// been implemented.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IpaPublicParameters {
    pub max_variables: usize,
}

/// Prover key for the IPA backend.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IpaProverKey {
    pub supported_variables: usize,
}

/// Verifier key for the IPA backend.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IpaVerifierKey {
    pub supported_variables: usize,
}

/// Opaque IPA commitment bytes.
///
/// A future implementation should replace `commitment_bytes` with a typed group
/// element representation and canonical serialization.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IpaCommitment {
    pub variables: usize,
    pub commitment_bytes: Vec<u8>,
}

/// Opaque IPA opening proof.
///
/// `claimed_value` is explicit because the PCS verifier returns the opened value
/// to the protocol verifier.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IpaOpening<F: PrimeField> {
    pub claimed_value: F,
    pub proof_bytes: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IpaBackendError {
    Shape(PcsShapeError),
    BackendNotImplemented,
}

impl From<PcsShapeError> for IpaBackendError {
    fn from(error: PcsShapeError) -> Self {
        Self::Shape(error)
    }
}

/// IPA backend marker type.
///
/// This intentionally does not implement fake cryptographic verification.
/// Unsupported operations return `BackendNotImplemented`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IpaBackend;

impl<F: PrimeField> MultilinearPcs<F> for IpaBackend {
    type PublicParameters = IpaPublicParameters;
    type ProverKey = IpaProverKey;
    type VerifierKey = IpaVerifierKey;
    type Commitment = IpaCommitment;
    type Opening = IpaOpening<F>;
    type Error = IpaBackendError;

    fn setup(max_variables: usize) -> Result<Self::PublicParameters, Self::Error> {
        Ok(IpaPublicParameters { max_variables })
    }

    fn trim(
        public_parameters: &Self::PublicParameters,
        supported_variables: usize,
    ) -> Result<(Self::ProverKey, Self::VerifierKey), Self::Error> {
        validate_supported_variables(public_parameters.max_variables, supported_variables)?;

        Ok((
            IpaProverKey {
                supported_variables,
            },
            IpaVerifierKey {
                supported_variables,
            },
        ))
    }

    fn commit(
        prover_key: &Self::ProverKey,
        polynomial: &Multilinear<F>,
    ) -> Result<Self::Commitment, Self::Error> {
        validate_supported_variables(prover_key.supported_variables, polynomial.variables())?;

        Err(IpaBackendError::BackendNotImplemented)
    }

    fn bind_commitment<T: ProofTranscript<F>>(commitment: &Self::Commitment, transcript: &mut T) {
        transcript.append_domain_separator(IPA_TRANSCRIPT_DOMAIN);
        transcript.append_u64(b"ipa-commitment-variables", commitment.variables as u64);
        transcript.append_bytes(b"ipa-commitment-bytes", &commitment.commitment_bytes);
    }

    fn open(
        prover_key: &Self::ProverKey,
        polynomial: &Multilinear<F>,
        point: &[F],
    ) -> Result<Self::Opening, Self::Error> {
        validate_supported_variables(prover_key.supported_variables, polynomial.variables())?;
        validate_opening_point(polynomial.variables(), point.len())?;

        Err(IpaBackendError::BackendNotImplemented)
    }

    fn verify(
        verifier_key: &Self::VerifierKey,
        commitment: &Self::Commitment,
        point: &[F],
        _opening: &Self::Opening,
    ) -> Result<F, Self::Error> {
        validate_supported_variables(verifier_key.supported_variables, commitment.variables)?;
        validate_opening_point(commitment.variables, point.len())?;

        Err(IpaBackendError::BackendNotImplemented)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ark_bls12_381::Fr;

    fn polynomial_with_variables(variables: usize) -> Multilinear<Fr> {
        let len = 1usize << variables;
        Multilinear::new((0..len as u64).map(Fr::from).collect()).unwrap()
    }

    #[test]
    fn setup_and_trim_produce_separate_key_shapes() {
        let pp = <IpaBackend as MultilinearPcs<Fr>>::setup(8).unwrap();
        let (pk, vk) = <IpaBackend as MultilinearPcs<Fr>>::trim(&pp, 5).unwrap();

        assert_eq!(pp.max_variables, 8);
        assert_eq!(pk.supported_variables, 5);
        assert_eq!(vk.supported_variables, 5);
    }

    #[test]
    fn trim_rejects_unsupported_variable_count() {
        let pp = <IpaBackend as MultilinearPcs<Fr>>::setup(4).unwrap();

        assert_eq!(
            <IpaBackend as MultilinearPcs<Fr>>::trim(&pp, 5),
            Err(IpaBackendError::Shape(
                PcsShapeError::UnsupportedVariableCount {
                    requested: 5,
                    supported: 4
                }
            ))
        );
    }

    #[test]
    fn commit_rejects_polynomial_above_key_capacity() {
        let pp = <IpaBackend as MultilinearPcs<Fr>>::setup(4).unwrap();
        let (pk, _vk) = <IpaBackend as MultilinearPcs<Fr>>::trim(&pp, 2).unwrap();
        let polynomial = polynomial_with_variables(3);

        assert_eq!(
            <IpaBackend as MultilinearPcs<Fr>>::commit(&pk, &polynomial),
            Err(IpaBackendError::Shape(
                PcsShapeError::UnsupportedVariableCount {
                    requested: 3,
                    supported: 2
                }
            ))
        );
    }

    #[test]
    fn commit_does_not_fake_success() {
        let pp = <IpaBackend as MultilinearPcs<Fr>>::setup(4).unwrap();
        let (pk, _vk) = <IpaBackend as MultilinearPcs<Fr>>::trim(&pp, 3).unwrap();
        let polynomial = polynomial_with_variables(3);

        assert_eq!(
            <IpaBackend as MultilinearPcs<Fr>>::commit(&pk, &polynomial),
            Err(IpaBackendError::BackendNotImplemented)
        );
    }

    #[test]
    fn open_rejects_wrong_point_length_before_backend_work() {
        let pp = <IpaBackend as MultilinearPcs<Fr>>::setup(4).unwrap();
        let (pk, _vk) = <IpaBackend as MultilinearPcs<Fr>>::trim(&pp, 3).unwrap();
        let polynomial = polynomial_with_variables(3);

        assert_eq!(
            <IpaBackend as MultilinearPcs<Fr>>::open(&pk, &polynomial, &[Fr::from(1), Fr::from(2)]),
            Err(IpaBackendError::Shape(PcsShapeError::PointLengthMismatch {
                expected: 3,
                actual: 2
            }))
        );
    }

    #[test]
    fn verify_rejects_commitment_above_key_capacity() {
        let pp = <IpaBackend as MultilinearPcs<Fr>>::setup(4).unwrap();
        let (_pk, vk) = <IpaBackend as MultilinearPcs<Fr>>::trim(&pp, 2).unwrap();

        let commitment = IpaCommitment {
            variables: 3,
            commitment_bytes: vec![1, 2, 3],
        };

        let opening = IpaOpening {
            claimed_value: Fr::from(7),
            proof_bytes: vec![4, 5, 6],
        };

        assert_eq!(
            <IpaBackend as MultilinearPcs<Fr>>::verify(
                &vk,
                &commitment,
                &[Fr::from(1), Fr::from(2), Fr::from(3)],
                &opening
            ),
            Err(IpaBackendError::Shape(
                PcsShapeError::UnsupportedVariableCount {
                    requested: 3,
                    supported: 2
                }
            ))
        );
    }

    #[test]
    fn verify_does_not_fake_success() {
        let pp = <IpaBackend as MultilinearPcs<Fr>>::setup(4).unwrap();
        let (_pk, vk) = <IpaBackend as MultilinearPcs<Fr>>::trim(&pp, 3).unwrap();

        let commitment = IpaCommitment {
            variables: 3,
            commitment_bytes: vec![1, 2, 3],
        };

        let opening = IpaOpening {
            claimed_value: Fr::from(7),
            proof_bytes: vec![4, 5, 6],
        };

        assert_eq!(
            <IpaBackend as MultilinearPcs<Fr>>::verify(
                &vk,
                &commitment,
                &[Fr::from(1), Fr::from(2), Fr::from(3)],
                &opening
            ),
            Err(IpaBackendError::BackendNotImplemented)
        );
    }
}
