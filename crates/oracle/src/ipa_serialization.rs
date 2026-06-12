use ark_ff::PrimeField;

use crate::ipa_proof::{validate_ipa_opening_proof_shape, IpaOpeningProof, IpaProofShapeError};
use crate::ipa_transcript::{IpaTranscriptError, IpaTranscriptRound};

const MAGIC: &[u8] = b"SL-IPA-PROOF1";
const MAX_ENCODED_IPA_VARIABLES: usize = 64;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IpaProofCodecError {
    InvalidMagic,
    Truncated,
    TrailingBytes,
    InvalidFieldElement,
    LengthOverflow,
    InvalidProofShape(IpaProofShapeError),
}

impl From<IpaProofShapeError> for IpaProofCodecError {
    fn from(error: IpaProofShapeError) -> Self {
        Self::InvalidProofShape(error)
    }
}

impl From<IpaTranscriptError> for IpaProofCodecError {
    fn from(error: IpaTranscriptError) -> Self {
        Self::InvalidProofShape(IpaProofShapeError::Transcript(error))
    }
}

fn usize_to_u64(value: usize) -> Result<u64, IpaProofCodecError> {
    value
        .try_into()
        .map_err(|_| IpaProofCodecError::LengthOverflow)
}

fn u64_to_usize(value: u64) -> Result<usize, IpaProofCodecError> {
    value
        .try_into()
        .map_err(|_| IpaProofCodecError::LengthOverflow)
}

fn validate_decoded_ipa_variable_count(value: usize) -> Result<(), IpaProofCodecError> {
    if value > MAX_ENCODED_IPA_VARIABLES {
        return Err(IpaProofCodecError::LengthOverflow);
    }

    Ok(())
}

fn validate_decoded_ipa_round_count(
    variables: usize,
    round_count: usize,
) -> Result<(), IpaProofCodecError> {
    validate_decoded_ipa_variable_count(variables)?;
    validate_decoded_ipa_variable_count(round_count)?;

    if round_count != variables {
        return Err(IpaProofCodecError::InvalidProofShape(
            IpaProofShapeError::Transcript(IpaTranscriptError::RoundCountMismatch {
                expected: variables,
                actual: round_count,
            }),
        ));
    }

    Ok(())
}

fn push_u64(out: &mut Vec<u8>, value: u64) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn read_exact<'a>(
    input: &'a [u8],
    cursor: &mut usize,
    len: usize,
) -> Result<&'a [u8], IpaProofCodecError> {
    let end = cursor
        .checked_add(len)
        .ok_or(IpaProofCodecError::LengthOverflow)?;

    if end > input.len() {
        return Err(IpaProofCodecError::Truncated);
    }

    let bytes = &input[*cursor..end];
    *cursor = end;
    Ok(bytes)
}

fn read_u64(input: &[u8], cursor: &mut usize) -> Result<u64, IpaProofCodecError> {
    let bytes = read_exact(input, cursor, 8)?;
    let mut array = [0u8; 8];
    array.copy_from_slice(bytes);
    Ok(u64::from_le_bytes(array))
}

fn field_bytes_len<F: PrimeField>() -> usize {
    F::MODULUS_BIT_SIZE.div_ceil(8) as usize
}

fn write_field<F: PrimeField>(out: &mut Vec<u8>, value: &F) {
    let len = field_bytes_len::<F>();
    let mut bytes = vec![0u8; len];
    let bigint = value.into_bigint();
    let limbs = bigint.as_ref();

    for (limb_index, limb) in limbs.iter().enumerate() {
        let limb_bytes = limb.to_le_bytes();
        let start = limb_index * 8;

        if start >= len {
            break;
        }

        let take = core::cmp::min(8, len - start);
        bytes[start..start + take].copy_from_slice(&limb_bytes[..take]);
    }

    out.extend_from_slice(&bytes);
}

fn read_field<F: PrimeField>(input: &[u8], cursor: &mut usize) -> Result<F, IpaProofCodecError> {
    let len = field_bytes_len::<F>();
    let bytes = read_exact(input, cursor, len)?;

    let decoded = F::from_le_bytes_mod_order(bytes);

    let mut canonical = Vec::with_capacity(len);
    write_field(&mut canonical, &decoded);

    if canonical.as_slice() != bytes {
        return Err(IpaProofCodecError::InvalidFieldElement);
    }

    Ok(decoded)
}

fn write_bytes(out: &mut Vec<u8>, bytes: &[u8]) -> Result<(), IpaProofCodecError> {
    push_u64(out, usize_to_u64(bytes.len())?);
    out.extend_from_slice(bytes);
    Ok(())
}

fn read_bytes(input: &[u8], cursor: &mut usize) -> Result<Vec<u8>, IpaProofCodecError> {
    let len = u64_to_usize(read_u64(input, cursor)?)?;
    Ok(read_exact(input, cursor, len)?.to_vec())
}

pub fn encode_ipa_opening_proof<F: PrimeField>(
    proof: &IpaOpeningProof<F>,
) -> Result<Vec<u8>, IpaProofCodecError> {
    validate_ipa_opening_proof_shape(proof)?;

    let mut out = Vec::new();

    out.extend_from_slice(MAGIC);
    push_u64(&mut out, F::MODULUS_BIT_SIZE as u64);
    push_u64(&mut out, usize_to_u64(proof.variables)?);

    write_field(&mut out, &proof.claimed_value);

    push_u64(&mut out, usize_to_u64(proof.rounds.len())?);

    for round in &proof.rounds {
        IpaTranscriptRound::new(
            round.round_index,
            round.left_commitment_bytes.clone(),
            round.right_commitment_bytes.clone(),
        )?;

        push_u64(&mut out, usize_to_u64(round.round_index)?);
        write_bytes(&mut out, &round.left_commitment_bytes)?;
        write_bytes(&mut out, &round.right_commitment_bytes)?;
    }

    write_field(&mut out, &proof.final_polynomial_scalar);
    write_field(&mut out, &proof.final_evaluation_basis_scalar);
    write_bytes(&mut out, &proof.final_commitment_bytes)?;

    Ok(out)
}

pub fn decode_ipa_opening_proof<F: PrimeField>(
    input: &[u8],
) -> Result<IpaOpeningProof<F>, IpaProofCodecError> {
    let mut cursor = 0usize;

    let magic = read_exact(input, &mut cursor, MAGIC.len())?;
    if magic != MAGIC {
        return Err(IpaProofCodecError::InvalidMagic);
    }

    let modulus_bits = read_u64(input, &mut cursor)?;
    if modulus_bits != F::MODULUS_BIT_SIZE as u64 {
        return Err(IpaProofCodecError::InvalidFieldElement);
    }

    let variables = u64_to_usize(read_u64(input, &mut cursor)?)?;
    validate_decoded_ipa_variable_count(variables)?;

    let claimed_value = read_field(input, &mut cursor)?;

    let round_count = u64_to_usize(read_u64(input, &mut cursor)?)?;
    validate_decoded_ipa_round_count(variables, round_count)?;

    let mut rounds = Vec::with_capacity(round_count);

    for _ in 0..round_count {
        let round_index = u64_to_usize(read_u64(input, &mut cursor)?)?;
        let left_commitment_bytes = read_bytes(input, &mut cursor)?;
        let right_commitment_bytes = read_bytes(input, &mut cursor)?;

        rounds.push(IpaTranscriptRound::new(
            round_index,
            left_commitment_bytes,
            right_commitment_bytes,
        )?);
    }

    let final_polynomial_scalar = read_field(input, &mut cursor)?;
    let final_evaluation_basis_scalar = read_field(input, &mut cursor)?;
    let final_commitment_bytes = read_bytes(input, &mut cursor)?;

    if cursor != input.len() {
        return Err(IpaProofCodecError::TrailingBytes);
    }

    Ok(IpaOpeningProof::new(
        variables,
        claimed_value,
        rounds,
        final_polynomial_scalar,
        final_evaluation_basis_scalar,
        final_commitment_bytes,
    )?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ark_bls12_381::Fr;

    fn round(index: usize) -> IpaTranscriptRound {
        IpaTranscriptRound::new(
            index,
            vec![index as u8 + 1, index as u8 + 11],
            vec![index as u8 + 2, index as u8 + 12],
        )
        .unwrap()
    }

    fn proof() -> IpaOpeningProof<Fr> {
        IpaOpeningProof::new(
            2,
            Fr::from(9),
            vec![round(0), round(1)],
            Fr::from(7),
            Fr::from(8),
            vec![1, 2, 3],
        )
        .unwrap()
    }

    #[test]
    fn ipa_opening_proof_roundtrip_is_canonical() {
        let proof = proof();

        let encoded = encode_ipa_opening_proof(&proof).unwrap();
        let decoded = decode_ipa_opening_proof::<Fr>(&encoded).unwrap();
        let reencoded = encode_ipa_opening_proof(&decoded).unwrap();

        assert_eq!(proof, decoded);
        assert_eq!(encoded, reencoded);
    }

    #[test]
    fn rejects_wrong_magic() {
        let mut encoded = encode_ipa_opening_proof(&proof()).unwrap();

        encoded[0] = b'X';

        assert_eq!(
            decode_ipa_opening_proof::<Fr>(&encoded),
            Err(IpaProofCodecError::InvalidMagic)
        );
    }

    #[test]
    fn rejects_truncated_input() {
        let mut encoded = encode_ipa_opening_proof(&proof()).unwrap();

        encoded.pop();

        assert_eq!(
            decode_ipa_opening_proof::<Fr>(&encoded),
            Err(IpaProofCodecError::Truncated)
        );
    }

    #[test]
    fn rejects_trailing_bytes() {
        let mut encoded = encode_ipa_opening_proof(&proof()).unwrap();

        encoded.push(0);

        assert_eq!(
            decode_ipa_opening_proof::<Fr>(&encoded),
            Err(IpaProofCodecError::TrailingBytes)
        );
    }

    #[test]
    fn rejects_empty_final_commitment() {
        let proof = proof();
        let mut encoded = encode_ipa_opening_proof(&proof).unwrap();

        let field_len = field_bytes_len::<Fr>();
        let mut cursor = MAGIC.len() + 8 + 8 + field_len + 8;

        for round in &proof.rounds {
            cursor += 8;
            cursor += 8 + round.left_commitment_bytes.len();
            cursor += 8 + round.right_commitment_bytes.len();
        }

        cursor += field_len;
        cursor += field_len;

        encoded[cursor..cursor + 8].copy_from_slice(&0u64.to_le_bytes());
        encoded.truncate(cursor + 8);

        assert_eq!(
            decode_ipa_opening_proof::<Fr>(&encoded),
            Err(IpaProofCodecError::InvalidProofShape(
                IpaProofShapeError::EmptyFinalCommitment
            ))
        );
    }
    #[test]
    fn rejects_fuzzed_oversized_round_count_without_panic() {
        let data = [
            83u8, 76, 45, 73, 80, 65, 45, 80, 82, 79, 79, 70, 49, 255, 0, 0, 0, 0, 0, 0, 0, 255,
            255, 255, 255, 255, 255, 162, 162, 162, 162, 162, 162, 162, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 255, 255,
            255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
            255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 10,
        ];

        assert_eq!(
            decode_ipa_opening_proof::<Fr>(&data),
            Err(IpaProofCodecError::LengthOverflow)
        );
    }
}
