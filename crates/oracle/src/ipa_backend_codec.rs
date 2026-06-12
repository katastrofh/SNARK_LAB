use ark_ff::PrimeField;

use crate::ipa_backend_integration::IpaIntegratedOpening;
use crate::ipa_serialization::{
    decode_ipa_opening_proof, encode_ipa_opening_proof, IpaProofCodecError,
};

const MAGIC: &[u8] = b"SL-IPA-BACKEND-OPEN1";

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IpaBackendOpeningCodecError<F: PrimeField> {
    InvalidMagic,
    InvalidField,
    Truncated,
    TrailingBytes,
    LengthOverflow,
    ClaimedValueMismatch {
        opening_claimed: F,
        proof_claimed: F,
    },
    ProofCodec(IpaProofCodecError),
}

impl<F: PrimeField> From<IpaProofCodecError> for IpaBackendOpeningCodecError<F> {
    fn from(error: IpaProofCodecError) -> Self {
        Self::ProofCodec(error)
    }
}

fn usize_to_u64<F: PrimeField>(value: usize) -> Result<u64, IpaBackendOpeningCodecError<F>> {
    value
        .try_into()
        .map_err(|_| IpaBackendOpeningCodecError::LengthOverflow)
}

fn u64_to_usize<F: PrimeField>(value: u64) -> Result<usize, IpaBackendOpeningCodecError<F>> {
    value
        .try_into()
        .map_err(|_| IpaBackendOpeningCodecError::LengthOverflow)
}

fn push_u64(out: &mut Vec<u8>, value: u64) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn read_exact<'a, F: PrimeField>(
    input: &'a [u8],
    cursor: &mut usize,
    len: usize,
) -> Result<&'a [u8], IpaBackendOpeningCodecError<F>> {
    let end = cursor
        .checked_add(len)
        .ok_or(IpaBackendOpeningCodecError::LengthOverflow)?;

    if end > input.len() {
        return Err(IpaBackendOpeningCodecError::Truncated);
    }

    let bytes = &input[*cursor..end];
    *cursor = end;
    Ok(bytes)
}

fn read_u64<F: PrimeField>(
    input: &[u8],
    cursor: &mut usize,
) -> Result<u64, IpaBackendOpeningCodecError<F>> {
    let bytes = read_exact::<F>(input, cursor, 8)?;
    let mut array = [0u8; 8];
    array.copy_from_slice(bytes);
    Ok(u64::from_le_bytes(array))
}

fn write_bytes<F: PrimeField>(
    out: &mut Vec<u8>,
    bytes: &[u8],
) -> Result<(), IpaBackendOpeningCodecError<F>> {
    push_u64(out, usize_to_u64::<F>(bytes.len())?);
    out.extend_from_slice(bytes);
    Ok(())
}

fn read_bytes<F: PrimeField>(
    input: &[u8],
    cursor: &mut usize,
) -> Result<Vec<u8>, IpaBackendOpeningCodecError<F>> {
    let len = u64_to_usize::<F>(read_u64::<F>(input, cursor)?)?;
    Ok(read_exact::<F>(input, cursor, len)?.to_vec())
}

/// Encode a public integrated IPA opening.
///
/// This intentionally serializes only:
///
/// - the opening claimed value as already embedded in the proof,
/// - the IPA opening proof.
///
/// It does not serialize `IpaIntegratedCommitmentWitness`, and therefore does
/// not expose the prover's blinding scalar.
pub fn encode_ipa_integrated_opening<F: PrimeField>(
    opening: &IpaIntegratedOpening<F>,
) -> Result<Vec<u8>, IpaBackendOpeningCodecError<F>> {
    if opening.claimed_value != opening.proof.claimed_value {
        return Err(IpaBackendOpeningCodecError::ClaimedValueMismatch {
            opening_claimed: opening.claimed_value,
            proof_claimed: opening.proof.claimed_value,
        });
    }

    let proof_bytes = encode_ipa_opening_proof(&opening.proof)?;

    let mut out = Vec::new();
    out.extend_from_slice(MAGIC);
    push_u64(&mut out, F::MODULUS_BIT_SIZE as u64);
    write_bytes::<F>(&mut out, &proof_bytes)?;

    Ok(out)
}

/// Decode a public integrated IPA opening.
///
/// The claimed value is reconstructed from the canonical IPA proof payload.
pub fn decode_ipa_integrated_opening<F: PrimeField>(
    input: &[u8],
) -> Result<IpaIntegratedOpening<F>, IpaBackendOpeningCodecError<F>> {
    let mut cursor = 0usize;

    let magic = read_exact::<F>(input, &mut cursor, MAGIC.len())?;
    if magic != MAGIC {
        return Err(IpaBackendOpeningCodecError::InvalidMagic);
    }

    let modulus_bits = read_u64::<F>(input, &mut cursor)?;
    if modulus_bits != F::MODULUS_BIT_SIZE as u64 {
        return Err(IpaBackendOpeningCodecError::InvalidField);
    }

    let proof_bytes = read_bytes::<F>(input, &mut cursor)?;

    if cursor != input.len() {
        return Err(IpaBackendOpeningCodecError::TrailingBytes);
    }

    let proof = decode_ipa_opening_proof::<F>(&proof_bytes)?;
    let claimed_value = proof.claimed_value;

    Ok(IpaIntegratedOpening {
        claimed_value,
        proof,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use ark_bls12_381::{Fr, G1Projective};
    use ark_ec::PrimeGroup;
    use multilinear::Multilinear;
    use snark_lab_transcript::MerlinTranscript;

    use crate::ipa::IpaCommitment;
    use crate::ipa_backend_integration::{
        commit_ipa_backend, open_ipa_backend, trim_ipa_integrated_keys, verify_ipa_backend,
        IpaIntegratedVerifierKey,
    };
    use crate::ipa_curve::{IpaCurveGeneratorBasis, IpaCurvePoint};
    use crate::ipa_generators::expected_ipa_generator_count;

    #[derive(Clone)]
    struct Fixture {
        verifier_key: IpaIntegratedVerifierKey<G1Projective>,
        commitment: IpaCommitment,
        point: Vec<Fr>,
        opening: IpaIntegratedOpening<Fr>,
    }

    fn point_generator(seed: u64) -> IpaCurvePoint<G1Projective> {
        IpaCurvePoint::from_projective(G1Projective::generator() * Fr::from(seed)).unwrap()
    }

    fn basis(variables: usize) -> IpaCurveGeneratorBasis<G1Projective> {
        let count = expected_ipa_generator_count(variables).unwrap();

        IpaCurveGeneratorBasis::new(
            variables,
            (0..count)
                .map(|index| point_generator(index as u64 + 1))
                .collect(),
            (0..count)
                .map(|index| point_generator(index as u64 + 100))
                .collect(),
            point_generator(999),
        )
        .unwrap()
    }

    fn padding_polynomial(variables: usize) -> Vec<IpaCurvePoint<G1Projective>> {
        let original_len = expected_ipa_generator_count(variables).unwrap();
        let extended_len = expected_ipa_generator_count(variables + 1).unwrap();

        (0..(extended_len - original_len - 1))
            .map(|index| point_generator(index as u64 + 2000))
            .collect()
    }

    fn padding_evaluation(variables: usize) -> Vec<IpaCurvePoint<G1Projective>> {
        let original_len = expected_ipa_generator_count(variables).unwrap();
        let extended_len = expected_ipa_generator_count(variables + 1).unwrap();

        (0..(extended_len - original_len))
            .map(|index| point_generator(index as u64 + 3000))
            .collect()
    }

    fn polynomial(values: &[u64]) -> Multilinear<Fr> {
        Multilinear::new(values.iter().copied().map(Fr::from).collect()).unwrap()
    }

    fn fixture() -> Fixture {
        let polynomial = polynomial(&[2, 3, 5, 7]);
        let variables = polynomial.variables();
        let point = vec![Fr::from(3), Fr::from(3)];

        let (prover_key, verifier_key) = trim_ipa_integrated_keys(
            basis(variables),
            point_generator(5000),
            padding_polynomial(variables),
            padding_evaluation(variables),
            point_generator(9000),
        )
        .unwrap();

        let witness = commit_ipa_backend(&prover_key, &polynomial, Fr::from(9)).unwrap();

        let mut transcript = MerlinTranscript::new(b"ipa-backend-codec-test");
        let opening =
            open_ipa_backend(&prover_key, &witness, &polynomial, &point, &mut transcript).unwrap();

        Fixture {
            verifier_key,
            commitment: witness.commitment,
            point,
            opening,
        }
    }

    fn verify_fixture(fixture: &Fixture, opening: &IpaIntegratedOpening<Fr>) {
        let mut transcript = MerlinTranscript::new(b"ipa-backend-codec-test");

        verify_ipa_backend(
            &fixture.verifier_key,
            &fixture.commitment,
            &fixture.point,
            opening,
            &mut transcript,
        )
        .unwrap();
    }

    #[test]
    fn integrated_opening_roundtrip_is_canonical_and_verifies() {
        let fixture = fixture();

        let encoded = encode_ipa_integrated_opening(&fixture.opening).unwrap();
        let decoded = decode_ipa_integrated_opening::<Fr>(&encoded).unwrap();
        let reencoded = encode_ipa_integrated_opening(&decoded).unwrap();

        assert_eq!(fixture.opening, decoded);
        assert_eq!(encoded, reencoded);

        verify_fixture(&fixture, &decoded);
    }

    #[test]
    fn encoder_rejects_claimed_value_mismatch() {
        let mut opening = fixture().opening;
        opening.claimed_value += Fr::from(1);

        assert_eq!(
            encode_ipa_integrated_opening(&opening),
            Err(IpaBackendOpeningCodecError::ClaimedValueMismatch {
                opening_claimed: opening.claimed_value,
                proof_claimed: opening.proof.claimed_value,
            })
        );
    }

    #[test]
    fn decoder_rejects_wrong_magic() {
        let mut encoded = encode_ipa_integrated_opening(&fixture().opening).unwrap();
        encoded[0] = b'X';

        assert_eq!(
            decode_ipa_integrated_opening::<Fr>(&encoded),
            Err(IpaBackendOpeningCodecError::InvalidMagic)
        );
    }

    #[test]
    fn decoder_rejects_truncated_input() {
        let mut encoded = encode_ipa_integrated_opening(&fixture().opening).unwrap();
        encoded.pop();

        assert_eq!(
            decode_ipa_integrated_opening::<Fr>(&encoded),
            Err(IpaBackendOpeningCodecError::Truncated)
        );
    }

    #[test]
    fn decoder_rejects_trailing_bytes() {
        let mut encoded = encode_ipa_integrated_opening(&fixture().opening).unwrap();
        encoded.push(0);

        assert_eq!(
            decode_ipa_integrated_opening::<Fr>(&encoded),
            Err(IpaBackendOpeningCodecError::TrailingBytes)
        );
    }

    #[test]
    fn decoder_rejects_corrupt_inner_proof() {
        let mut encoded = encode_ipa_integrated_opening(&fixture().opening).unwrap();
        let proof_start = MAGIC.len() + 8 + 8;
        encoded[proof_start] = b'X';

        assert_eq!(
            decode_ipa_integrated_opening::<Fr>(&encoded),
            Err(IpaBackendOpeningCodecError::ProofCodec(
                IpaProofCodecError::InvalidMagic
            ))
        );
    }

    #[test]
    fn decoded_tampered_opening_does_not_verify() {
        let fixture = fixture();
        let mut encoded = encode_ipa_integrated_opening(&fixture.opening).unwrap();

        let proof_start = MAGIC.len() + 8 + 8;
        let first_field_byte = proof_start + b"SL-IPA-PROOF1".len() + 8 + 8;
        encoded[first_field_byte] ^= 1;

        if let Ok(decoded_opening) = decode_ipa_integrated_opening::<Fr>(&encoded) {
            let mut transcript = MerlinTranscript::new(b"ipa-backend-codec-test");

            assert!(verify_ipa_backend(
                &fixture.verifier_key,
                &fixture.commitment,
                &fixture.point,
                &decoded_opening,
                &mut transcript,
            )
            .is_err());
        }
    }
}
