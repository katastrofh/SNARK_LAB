use ark_ec::{AffineRepr, CurveGroup};
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use std::{fs, io, path::Path};

use crate::ipa_curve::{IpaCurveGeneratorBasis, IpaCurvePoint, IpaCurvePointError};
use crate::ipa_srs_provenance::{
    validate_ipa_srs_provenance, IpaSrsProvenance, IpaSrsProvenanceError, IpaSrsSource,
    IpaVerifiedSrs,
};

const MAGIC: &[u8] = b"SL-IPA-SRS-FILE1";
const SOURCE_EXTERNAL_TRUSTED_SETUP: u8 = 1;
const SOURCE_HASH_TO_CURVE_DERIVATION: u8 = 2;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IpaSrsFileError {
    InvalidMagic,
    UnknownSourceTag(u8),
    NonProductionSource,
    InvalidUtf8,
    Truncated,
    TrailingBytes,
    LengthOverflow,
    EmptyPointEncoding,
    Io {
        kind: io::ErrorKind,
        message: String,
    },
    Curve(IpaCurvePointError),
    Provenance(IpaSrsProvenanceError),
}

impl From<IpaCurvePointError> for IpaSrsFileError {
    fn from(error: IpaCurvePointError) -> Self {
        Self::Curve(error)
    }
}

impl From<IpaSrsProvenanceError> for IpaSrsFileError {
    fn from(error: IpaSrsProvenanceError) -> Self {
        Self::Provenance(error)
    }
}

impl From<io::Error> for IpaSrsFileError {
    fn from(error: io::Error) -> Self {
        Self::Io {
            kind: error.kind(),
            message: error.to_string(),
        }
    }
}

fn usize_to_u64(value: usize) -> Result<u64, IpaSrsFileError> {
    value
        .try_into()
        .map_err(|_| IpaSrsFileError::LengthOverflow)
}

fn u64_to_usize(value: u64) -> Result<usize, IpaSrsFileError> {
    value
        .try_into()
        .map_err(|_| IpaSrsFileError::LengthOverflow)
}

fn push_u64(out: &mut Vec<u8>, value: u64) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn write_bytes(out: &mut Vec<u8>, bytes: &[u8]) -> Result<(), IpaSrsFileError> {
    push_u64(out, usize_to_u64(bytes.len())?);
    out.extend_from_slice(bytes);
    Ok(())
}

fn write_string(out: &mut Vec<u8>, value: &str) -> Result<(), IpaSrsFileError> {
    write_bytes(out, value.as_bytes())
}

fn write_digest(out: &mut Vec<u8>, digest: &[u8; 32]) {
    out.extend_from_slice(digest);
}

fn read_exact<'a>(
    input: &'a [u8],
    cursor: &mut usize,
    len: usize,
) -> Result<&'a [u8], IpaSrsFileError> {
    let end = cursor
        .checked_add(len)
        .ok_or(IpaSrsFileError::LengthOverflow)?;

    if end > input.len() {
        return Err(IpaSrsFileError::Truncated);
    }

    let bytes = &input[*cursor..end];
    *cursor = end;
    Ok(bytes)
}

fn read_u8(input: &[u8], cursor: &mut usize) -> Result<u8, IpaSrsFileError> {
    Ok(read_exact(input, cursor, 1)?[0])
}

fn read_u64(input: &[u8], cursor: &mut usize) -> Result<u64, IpaSrsFileError> {
    let bytes = read_exact(input, cursor, 8)?;
    let mut out = [0u8; 8];
    out.copy_from_slice(bytes);
    Ok(u64::from_le_bytes(out))
}

fn read_bytes(input: &[u8], cursor: &mut usize) -> Result<Vec<u8>, IpaSrsFileError> {
    let len = u64_to_usize(read_u64(input, cursor)?)?;
    Ok(read_exact(input, cursor, len)?.to_vec())
}

fn read_string(input: &[u8], cursor: &mut usize) -> Result<String, IpaSrsFileError> {
    String::from_utf8(read_bytes(input, cursor)?).map_err(|_| IpaSrsFileError::InvalidUtf8)
}

fn read_digest(input: &[u8], cursor: &mut usize) -> Result<[u8; 32], IpaSrsFileError> {
    let bytes = read_exact(input, cursor, 32)?;
    let mut digest = [0u8; 32];
    digest.copy_from_slice(bytes);
    Ok(digest)
}

fn write_point<G>(out: &mut Vec<u8>, point: &IpaCurvePoint<G>) -> Result<(), IpaSrsFileError>
where
    G: CurveGroup,
    G::Affine: AffineRepr + CanonicalSerialize + CanonicalDeserialize,
{
    write_bytes(out, &point.to_compressed_bytes()?)
}

fn read_point<G>(input: &[u8], cursor: &mut usize) -> Result<IpaCurvePoint<G>, IpaSrsFileError>
where
    G: CurveGroup,
    G::Affine: AffineRepr + CanonicalSerialize + CanonicalDeserialize,
{
    let bytes = read_bytes(input, cursor)?;
    if bytes.is_empty() {
        return Err(IpaSrsFileError::EmptyPointEncoding);
    }

    Ok(IpaCurvePoint::from_compressed_bytes(&bytes)?)
}

fn write_source(out: &mut Vec<u8>, source: &IpaSrsSource) -> Result<(), IpaSrsFileError> {
    match source {
        IpaSrsSource::ExternalTrustedSetup {
            name,
            uri,
            artifact_sha256,
        } => {
            out.push(SOURCE_EXTERNAL_TRUSTED_SETUP);
            write_string(out, name)?;
            write_string(out, uri)?;
            write_digest(out, artifact_sha256);
            Ok(())
        }
        IpaSrsSource::HashToCurveDerivation {
            domain_separator,
            transcript_sha256,
        } => {
            out.push(SOURCE_HASH_TO_CURVE_DERIVATION);
            write_bytes(out, domain_separator)?;
            write_digest(out, transcript_sha256);
            Ok(())
        }
        IpaSrsSource::KnownDiscreteLogTestFixture => Err(IpaSrsFileError::NonProductionSource),
    }
}

fn read_source(input: &[u8], cursor: &mut usize) -> Result<IpaSrsSource, IpaSrsFileError> {
    match read_u8(input, cursor)? {
        SOURCE_EXTERNAL_TRUSTED_SETUP => Ok(IpaSrsSource::ExternalTrustedSetup {
            name: read_string(input, cursor)?,
            uri: read_string(input, cursor)?,
            artifact_sha256: read_digest(input, cursor)?,
        }),
        SOURCE_HASH_TO_CURVE_DERIVATION => Ok(IpaSrsSource::HashToCurveDerivation {
            domain_separator: read_bytes(input, cursor)?,
            transcript_sha256: read_digest(input, cursor)?,
        }),
        tag => Err(IpaSrsFileError::UnknownSourceTag(tag)),
    }
}

/// Encode a production-validated IPA SRS file.
///
/// This function accepts only `IpaVerifiedSrs`, so callers cannot serialize
/// unvalidated or known-discrete-log test fixture material through this API.
pub fn encode_ipa_srs_file<G>(verified: &IpaVerifiedSrs<G>) -> Result<Vec<u8>, IpaSrsFileError>
where
    G: CurveGroup,
    G::Affine: AffineRepr + CanonicalSerialize + CanonicalDeserialize,
{
    let provenance = verified.provenance();
    let basis = verified.basis();

    let mut out = Vec::new();
    out.extend_from_slice(MAGIC);

    push_u64(&mut out, usize_to_u64(provenance.max_variables)?);
    write_string(&mut out, &provenance.curve_id)?;
    write_source(&mut out, &provenance.source)?;
    write_digest(&mut out, &provenance.canonical_basis_sha256);

    push_u64(&mut out, usize_to_u64(basis.polynomial_generators.len())?);
    for generator in &basis.polynomial_generators {
        write_point(&mut out, generator)?;
    }

    push_u64(&mut out, usize_to_u64(basis.evaluation_generators.len())?);
    for generator in &basis.evaluation_generators {
        write_point(&mut out, generator)?;
    }

    write_point(&mut out, &basis.blinding_generator)?;

    Ok(out)
}

/// Decode and validate a canonical IPA SRS file from bytes.
///
/// Decoding is fail-closed: the returned object is already provenance-validated.
pub fn decode_ipa_srs_file<G>(input: &[u8]) -> Result<IpaVerifiedSrs<G>, IpaSrsFileError>
where
    G: CurveGroup,
    G::Affine: AffineRepr + CanonicalSerialize + CanonicalDeserialize,
{
    let mut cursor = 0usize;

    let magic = read_exact(input, &mut cursor, MAGIC.len())?;
    if magic != MAGIC {
        return Err(IpaSrsFileError::InvalidMagic);
    }

    let max_variables = u64_to_usize(read_u64(input, &mut cursor)?)?;
    let curve_id = read_string(input, &mut cursor)?;
    let source = read_source(input, &mut cursor)?;
    let canonical_basis_sha256 = read_digest(input, &mut cursor)?;

    let polynomial_len = u64_to_usize(read_u64(input, &mut cursor)?)?;
    let polynomial_generators = (0..polynomial_len)
        .map(|_| read_point(input, &mut cursor))
        .collect::<Result<Vec<_>, _>>()?;

    let evaluation_len = u64_to_usize(read_u64(input, &mut cursor)?)?;
    let evaluation_generators = (0..evaluation_len)
        .map(|_| read_point(input, &mut cursor))
        .collect::<Result<Vec<_>, _>>()?;

    let blinding_generator = read_point(input, &mut cursor)?;

    if cursor != input.len() {
        return Err(IpaSrsFileError::TrailingBytes);
    }

    let basis = IpaCurveGeneratorBasis::new(
        max_variables,
        polynomial_generators,
        evaluation_generators,
        blinding_generator,
    )?;

    let provenance = IpaSrsProvenance {
        curve_id,
        max_variables,
        source,
        canonical_basis_sha256,
    };

    Ok(validate_ipa_srs_provenance(basis, provenance)?)
}

/// Read, decode, and validate an IPA SRS file from disk.
pub fn read_ipa_srs_file<G, P>(path: P) -> Result<IpaVerifiedSrs<G>, IpaSrsFileError>
where
    G: CurveGroup,
    G::Affine: AffineRepr + CanonicalSerialize + CanonicalDeserialize,
    P: AsRef<Path>,
{
    let bytes = fs::read(path)?;
    decode_ipa_srs_file(&bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ark_bls12_381::{Fr, G1Projective};
    use ark_ec::PrimeGroup;

    use crate::ipa_generators::expected_ipa_generator_count;
    use crate::ipa_srs_provenance::{canonical_ipa_srs_digest, validate_ipa_srs_provenance};

    fn point(seed: u64) -> IpaCurvePoint<G1Projective> {
        IpaCurvePoint::from_projective(G1Projective::generator() * Fr::from(seed)).unwrap()
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

    fn verified_srs() -> IpaVerifiedSrs<G1Projective> {
        let basis = basis(2);
        let provenance = IpaSrsProvenance {
            curve_id: "BLS12-381-G1".to_string(),
            max_variables: basis.variables,
            source: IpaSrsSource::ExternalTrustedSetup {
                name: "example-audited-srs".to_string(),
                uri: "file:///opt/snark-lab/srs/ipa-g1-v1.bin".to_string(),
                artifact_sha256: nonzero_digest(7),
            },
            canonical_basis_sha256: canonical_ipa_srs_digest(&basis).unwrap(),
        };

        validate_ipa_srs_provenance(basis, provenance).unwrap()
    }

    fn source_tag_offset(encoded: &[u8]) -> usize {
        let mut cursor = MAGIC.len() + 8;
        let mut len = [0u8; 8];
        len.copy_from_slice(&encoded[cursor..cursor + 8]);
        cursor += 8 + u64::from_le_bytes(len) as usize;
        cursor
    }

    fn canonical_digest_offset_for_external(encoded: &[u8]) -> usize {
        let mut cursor = source_tag_offset(encoded);
        assert_eq!(encoded[cursor], SOURCE_EXTERNAL_TRUSTED_SETUP);
        cursor += 1;

        for _ in 0..2 {
            let mut len = [0u8; 8];
            len.copy_from_slice(&encoded[cursor..cursor + 8]);
            cursor += 8 + u64::from_le_bytes(len) as usize;
        }

        cursor + 32
    }

    #[test]
    fn srs_file_roundtrip_is_canonical_and_validated() {
        let verified = verified_srs();

        let encoded = encode_ipa_srs_file(&verified).unwrap();
        let decoded = decode_ipa_srs_file::<G1Projective>(&encoded).unwrap();
        let reencoded = encode_ipa_srs_file(&decoded).unwrap();

        assert_eq!(encoded, reencoded);
        assert_eq!(decoded.provenance(), verified.provenance());
        assert_eq!(decoded.basis(), verified.basis());
    }

    #[test]
    fn read_srs_file_from_disk_validates_loaded_material() {
        let verified = verified_srs();
        let encoded = encode_ipa_srs_file(&verified).unwrap();

        let path = std::env::temp_dir().join(format!(
            "snark-lab-ipa-srs-loader-{}.bin",
            std::process::id()
        ));

        std::fs::write(&path, encoded).unwrap();
        let decoded = read_ipa_srs_file::<G1Projective, _>(&path).unwrap();
        std::fs::remove_file(path).unwrap();

        assert_eq!(decoded.provenance(), verified.provenance());
        assert_eq!(decoded.basis(), verified.basis());
    }

    #[test]
    fn srs_file_decoder_rejects_wrong_magic() {
        let verified = verified_srs();
        let mut encoded = encode_ipa_srs_file(&verified).unwrap();
        encoded[0] = b'X';

        assert_eq!(
            decode_ipa_srs_file::<G1Projective>(&encoded),
            Err(IpaSrsFileError::InvalidMagic)
        );
    }

    #[test]
    fn srs_file_decoder_rejects_unknown_source_tag() {
        let verified = verified_srs();
        let mut encoded = encode_ipa_srs_file(&verified).unwrap();
        let offset = source_tag_offset(&encoded);
        encoded[offset] = 99;

        assert_eq!(
            decode_ipa_srs_file::<G1Projective>(&encoded),
            Err(IpaSrsFileError::UnknownSourceTag(99))
        );
    }

    #[test]
    fn srs_file_decoder_rejects_truncated_input() {
        let verified = verified_srs();
        let mut encoded = encode_ipa_srs_file(&verified).unwrap();
        encoded.pop();

        assert_eq!(
            decode_ipa_srs_file::<G1Projective>(&encoded),
            Err(IpaSrsFileError::Truncated)
        );
    }

    #[test]
    fn srs_file_decoder_rejects_trailing_bytes() {
        let verified = verified_srs();
        let mut encoded = encode_ipa_srs_file(&verified).unwrap();
        encoded.push(0);

        assert_eq!(
            decode_ipa_srs_file::<G1Projective>(&encoded),
            Err(IpaSrsFileError::TrailingBytes)
        );
    }

    #[test]
    fn srs_file_decoder_rejects_digest_mismatch() {
        let verified = verified_srs();
        let mut encoded = encode_ipa_srs_file(&verified).unwrap();
        let offset = canonical_digest_offset_for_external(&encoded);
        encoded[offset] ^= 1;

        assert!(matches!(
            decode_ipa_srs_file::<G1Projective>(&encoded),
            Err(IpaSrsFileError::Provenance(
                IpaSrsProvenanceError::DigestMismatch { .. }
            ))
        ));
    }

    #[test]
    fn srs_file_encoder_rejects_nonproduction_source_even_if_constructed_in_tests() {
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
}
