use ark_ec::{AffineRepr, CurveGroup};
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use sha2::{Digest, Sha256};

use crate::ipa_curve::{IpaCurveGeneratorBasis, IpaCurvePoint, IpaCurvePointError};

const SRS_DIGEST_DOMAIN: &[u8] = b"snark-lab/ipa-srs-canonical-digest/v1";

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IpaSrsSource {
    /// Externally supplied SRS artifact with an independently recorded artifact digest.
    ExternalTrustedSetup {
        name: String,
        uri: String,
        artifact_sha256: [u8; 32],
    },
    /// Generator basis derived by a hash-to-curve process outside this module.
    ///
    /// This module validates the resulting points and provenance metadata. It does
    /// not implement hash-to-curve derivation.
    HashToCurveDerivation {
        domain_separator: Vec<u8>,
        transcript_sha256: [u8; 32],
    },
    /// Test fixture source where the discrete logs are known to the test author.
    ///
    /// This source is intentionally rejected by production validation.
    KnownDiscreteLogTestFixture,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IpaSrsProvenance {
    pub curve_id: String,
    pub max_variables: usize,
    pub source: IpaSrsSource,
    pub canonical_basis_sha256: [u8; 32],
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IpaSrsProvenanceError {
    Curve(IpaCurvePointError),
    EmptyCurveId,
    EmptySourceField {
        field: &'static str,
    },
    ZeroDigest {
        field: &'static str,
    },
    NonProductionSource,
    VariableMismatch {
        basis_variables: usize,
        provenance_variables: usize,
    },
    LengthOverflow,
    DigestMismatch {
        expected: [u8; 32],
        actual: [u8; 32],
    },
}

impl From<IpaCurvePointError> for IpaSrsProvenanceError {
    fn from(error: IpaCurvePointError) -> Self {
        Self::Curve(error)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IpaVerifiedSrs<G: CurveGroup> {
    provenance: IpaSrsProvenance,
    basis: IpaCurveGeneratorBasis<G>,
}

impl<G: CurveGroup> IpaVerifiedSrs<G> {
    pub fn provenance(&self) -> &IpaSrsProvenance {
        &self.provenance
    }

    pub fn basis(&self) -> &IpaCurveGeneratorBasis<G> {
        &self.basis
    }

    pub fn into_parts(self) -> (IpaSrsProvenance, IpaCurveGeneratorBasis<G>) {
        (self.provenance, self.basis)
    }
}

fn is_zero_digest(digest: &[u8; 32]) -> bool {
    digest.iter().all(|byte| *byte == 0)
}

fn require_nonempty_text(field: &'static str, value: &str) -> Result<(), IpaSrsProvenanceError> {
    if value.trim().is_empty() {
        return Err(IpaSrsProvenanceError::EmptySourceField { field });
    }

    Ok(())
}

fn require_nonempty_bytes(field: &'static str, value: &[u8]) -> Result<(), IpaSrsProvenanceError> {
    if value.is_empty() {
        return Err(IpaSrsProvenanceError::EmptySourceField { field });
    }

    Ok(())
}

fn require_nonzero_digest(
    field: &'static str,
    digest: &[u8; 32],
) -> Result<(), IpaSrsProvenanceError> {
    if is_zero_digest(digest) {
        return Err(IpaSrsProvenanceError::ZeroDigest { field });
    }

    Ok(())
}

fn validate_source(source: &IpaSrsSource) -> Result<(), IpaSrsProvenanceError> {
    match source {
        IpaSrsSource::ExternalTrustedSetup {
            name,
            uri,
            artifact_sha256,
        } => {
            require_nonempty_text("external.name", name)?;
            require_nonempty_text("external.uri", uri)?;
            require_nonzero_digest("external.artifact_sha256", artifact_sha256)?;
            Ok(())
        }
        IpaSrsSource::HashToCurveDerivation {
            domain_separator,
            transcript_sha256,
        } => {
            require_nonempty_bytes("hash_to_curve.domain_separator", domain_separator)?;
            require_nonzero_digest("hash_to_curve.transcript_sha256", transcript_sha256)?;
            Ok(())
        }
        IpaSrsSource::KnownDiscreteLogTestFixture => {
            Err(IpaSrsProvenanceError::NonProductionSource)
        }
    }
}

fn hash_u64(hasher: &mut Sha256, value: u64) {
    hasher.update(value.to_le_bytes());
}

fn hash_len(hasher: &mut Sha256, value: usize) -> Result<(), IpaSrsProvenanceError> {
    let value = u64::try_from(value).map_err(|_| IpaSrsProvenanceError::LengthOverflow)?;
    hash_u64(hasher, value);
    Ok(())
}

fn hash_bytes(
    hasher: &mut Sha256,
    label: &[u8],
    bytes: &[u8],
) -> Result<(), IpaSrsProvenanceError> {
    hash_len(hasher, label.len())?;
    hasher.update(label);
    hash_len(hasher, bytes.len())?;
    hasher.update(bytes);
    Ok(())
}

fn hash_point<G>(
    hasher: &mut Sha256,
    label: &[u8],
    point: &IpaCurvePoint<G>,
) -> Result<(), IpaSrsProvenanceError>
where
    G: CurveGroup,
    G::Affine: AffineRepr + CanonicalSerialize + CanonicalDeserialize,
{
    let bytes = point.to_compressed_bytes()?;
    hash_bytes(hasher, label, &bytes)
}

/// Compute the canonical digest of a checked IPA generator basis.
///
/// This digest binds:
///
/// - digest domain version,
/// - variable count,
/// - polynomial generator compressed bytes,
/// - evaluation generator compressed bytes,
/// - blinding generator compressed bytes.
///
/// The digest does not claim randomness. It only makes the exact SRS material
/// auditable and reproducible.
pub fn canonical_ipa_srs_digest<G>(
    basis: &IpaCurveGeneratorBasis<G>,
) -> Result<[u8; 32], IpaSrsProvenanceError>
where
    G: CurveGroup,
    G::Affine: AffineRepr + CanonicalSerialize + CanonicalDeserialize,
{
    basis.validate()?;

    let mut hasher = Sha256::new();
    hasher.update(SRS_DIGEST_DOMAIN);
    hash_len(&mut hasher, basis.variables)?;

    for generator in &basis.polynomial_generators {
        hash_point(&mut hasher, b"polynomial", generator)?;
    }

    for generator in &basis.evaluation_generators {
        hash_point(&mut hasher, b"evaluation", generator)?;
    }

    hash_point(&mut hasher, b"blinding", &basis.blinding_generator)?;

    Ok(hasher.finalize().into())
}

/// Validate an externally supplied or hash-to-curve-derived IPA SRS.
///
/// This function fails closed for:
///
/// - known-discrete-log test fixture provenance,
/// - empty source metadata,
/// - zero source digests,
/// - basis/provenance variable mismatch,
/// - canonical basis digest mismatch,
/// - invalid, identity, duplicate, or wrong-count curve points.
pub fn validate_ipa_srs_provenance<G>(
    basis: IpaCurveGeneratorBasis<G>,
    provenance: IpaSrsProvenance,
) -> Result<IpaVerifiedSrs<G>, IpaSrsProvenanceError>
where
    G: CurveGroup,
    G::Affine: AffineRepr + CanonicalSerialize + CanonicalDeserialize,
{
    basis.validate()?;

    if provenance.curve_id.trim().is_empty() {
        return Err(IpaSrsProvenanceError::EmptyCurveId);
    }

    validate_source(&provenance.source)?;

    if basis.variables != provenance.max_variables {
        return Err(IpaSrsProvenanceError::VariableMismatch {
            basis_variables: basis.variables,
            provenance_variables: provenance.max_variables,
        });
    }

    let actual = canonical_ipa_srs_digest(&basis)?;
    if actual != provenance.canonical_basis_sha256 {
        return Err(IpaSrsProvenanceError::DigestMismatch {
            expected: provenance.canonical_basis_sha256,
            actual,
        });
    }

    Ok(IpaVerifiedSrs { provenance, basis })
}

#[cfg(test)]
mod tests {
    use super::*;
    use ark_bls12_381::G1Projective;
    use ark_ec::PrimeGroup;

    use crate::ipa_generators::expected_ipa_generator_count;

    fn point(seed: u64) -> IpaCurvePoint<G1Projective> {
        IpaCurvePoint::from_projective(G1Projective::generator() * ark_bls12_381::Fr::from(seed))
            .unwrap()
    }

    fn basis(variables: usize) -> IpaCurveGeneratorBasis<G1Projective> {
        let count = expected_ipa_generator_count(variables).unwrap();

        IpaCurveGeneratorBasis::new(
            variables,
            (0..count).map(|index| point(index as u64 + 1)).collect(),
            (0..count).map(|index| point(index as u64 + 100)).collect(),
            point(999),
        )
        .unwrap()
    }

    fn nonzero_digest(seed: u8) -> [u8; 32] {
        [seed; 32]
    }

    fn external_provenance(basis: &IpaCurveGeneratorBasis<G1Projective>) -> IpaSrsProvenance {
        IpaSrsProvenance {
            curve_id: "BLS12-381-G1".to_string(),
            max_variables: basis.variables,
            source: IpaSrsSource::ExternalTrustedSetup {
                name: "example-audited-srs".to_string(),
                uri: "file:///opt/snark-lab/srs/ipa-g1-v1.bin".to_string(),
                artifact_sha256: nonzero_digest(7),
            },
            canonical_basis_sha256: canonical_ipa_srs_digest(basis).unwrap(),
        }
    }

    #[test]
    fn validates_external_srs_with_matching_digest() {
        let basis = basis(2);
        let provenance = external_provenance(&basis);

        let verified = validate_ipa_srs_provenance(basis.clone(), provenance.clone()).unwrap();

        assert_eq!(verified.basis(), &basis);
        assert_eq!(verified.provenance(), &provenance);
    }

    #[test]
    fn validates_hash_to_curve_provenance_with_matching_digest() {
        let basis = basis(2);
        let provenance = IpaSrsProvenance {
            curve_id: "BLS12-381-G1".to_string(),
            max_variables: basis.variables,
            source: IpaSrsSource::HashToCurveDerivation {
                domain_separator: b"snark-lab-production-ipa-g1-v1".to_vec(),
                transcript_sha256: nonzero_digest(8),
            },
            canonical_basis_sha256: canonical_ipa_srs_digest(&basis).unwrap(),
        };

        assert!(validate_ipa_srs_provenance(basis, provenance).is_ok());
    }

    #[test]
    fn rejects_known_discrete_log_test_fixture_source() {
        let basis = basis(2);
        let provenance = IpaSrsProvenance {
            curve_id: "BLS12-381-G1".to_string(),
            max_variables: basis.variables,
            source: IpaSrsSource::KnownDiscreteLogTestFixture,
            canonical_basis_sha256: canonical_ipa_srs_digest(&basis).unwrap(),
        };

        assert_eq!(
            validate_ipa_srs_provenance(basis, provenance),
            Err(IpaSrsProvenanceError::NonProductionSource)
        );
    }

    #[test]
    fn rejects_digest_mismatch() {
        let basis = basis(2);
        let mut provenance = external_provenance(&basis);
        provenance.canonical_basis_sha256[0] ^= 1;

        assert!(matches!(
            validate_ipa_srs_provenance(basis, provenance),
            Err(IpaSrsProvenanceError::DigestMismatch { .. })
        ));
    }

    #[test]
    fn rejects_variable_mismatch() {
        let basis = basis(2);
        let mut provenance = external_provenance(&basis);
        provenance.max_variables = 3;

        assert_eq!(
            validate_ipa_srs_provenance(basis, provenance),
            Err(IpaSrsProvenanceError::VariableMismatch {
                basis_variables: 2,
                provenance_variables: 3,
            })
        );
    }

    #[test]
    fn rejects_empty_external_source_fields() {
        let basis = basis(2);
        let provenance = IpaSrsProvenance {
            curve_id: "BLS12-381-G1".to_string(),
            max_variables: basis.variables,
            source: IpaSrsSource::ExternalTrustedSetup {
                name: "".to_string(),
                uri: "file:///srs.bin".to_string(),
                artifact_sha256: nonzero_digest(9),
            },
            canonical_basis_sha256: canonical_ipa_srs_digest(&basis).unwrap(),
        };

        assert_eq!(
            validate_ipa_srs_provenance(basis, provenance),
            Err(IpaSrsProvenanceError::EmptySourceField {
                field: "external.name"
            })
        );
    }

    #[test]
    fn rejects_zero_artifact_digest() {
        let basis = basis(2);
        let provenance = IpaSrsProvenance {
            curve_id: "BLS12-381-G1".to_string(),
            max_variables: basis.variables,
            source: IpaSrsSource::ExternalTrustedSetup {
                name: "example-audited-srs".to_string(),
                uri: "file:///srs.bin".to_string(),
                artifact_sha256: [0u8; 32],
            },
            canonical_basis_sha256: canonical_ipa_srs_digest(&basis).unwrap(),
        };

        assert_eq!(
            validate_ipa_srs_provenance(basis, provenance),
            Err(IpaSrsProvenanceError::ZeroDigest {
                field: "external.artifact_sha256"
            })
        );
    }

    #[test]
    fn canonical_digest_changes_when_generator_changes() {
        let first = basis(2);
        let mut second = basis(2);
        second.polynomial_generators[0] = point(7_777);

        assert_ne!(
            canonical_ipa_srs_digest(&first).unwrap(),
            canonical_ipa_srs_digest(&second).unwrap()
        );
    }
}
