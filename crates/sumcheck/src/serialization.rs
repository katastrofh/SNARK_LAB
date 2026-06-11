use ark_ff::PrimeField;

use crate::{Proof, RoundPolynomial};

const MAGIC: &[u8; 8] = b"SL-SUM1";

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SumcheckCodecError {
    InvalidMagic,
    Truncated,
    TrailingBytes,
    InvalidFieldElement,
    LengthOverflow,
}

fn push_u64(out: &mut Vec<u8>, value: u64) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn read_exact<'a>(
    input: &'a [u8],
    cursor: &mut usize,
    len: usize,
) -> Result<&'a [u8], SumcheckCodecError> {
    let end = cursor.checked_add(len).ok_or(SumcheckCodecError::LengthOverflow)?;
    if end > input.len() {
        return Err(SumcheckCodecError::Truncated);
    }
    let bytes = &input[*cursor..end];
    *cursor = end;
    Ok(bytes)
}

fn read_u64(input: &[u8], cursor: &mut usize) -> Result<u64, SumcheckCodecError> {
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

fn read_field<F: PrimeField>(
    input: &[u8],
    cursor: &mut usize,
) -> Result<F, SumcheckCodecError> {
    let len = field_bytes_len::<F>();
    let bytes = read_exact(input, cursor, len)?;
    F::from_le_bytes_mod_order(bytes)
        .into()
        .ok_or(SumcheckCodecError::InvalidFieldElement)
}

pub fn encode_proof<F: PrimeField>(proof: &Proof<F>) -> Result<Vec<u8>, SumcheckCodecError> {
    let rounds: u64 = proof
        .round_polynomials
        .len()
        .try_into()
        .map_err(|_| SumcheckCodecError::LengthOverflow)?;

    let mut out = Vec::new();
    out.extend_from_slice(MAGIC);
    push_u64(&mut out, F::MODULUS_BIT_SIZE as u64);
    push_u64(&mut out, rounds);

    for round in &proof.round_polynomials {
        write_field(&mut out, &round.evaluation_at_zero);
        write_field(&mut out, &round.evaluation_at_one);
    }

    write_field(&mut out, &proof.final_evaluation);

    Ok(out)
}

pub fn decode_proof<F: PrimeField>(input: &[u8]) -> Result<Proof<F>, SumcheckCodecError> {
    let mut cursor = 0usize;

    let magic = read_exact(input, &mut cursor, MAGIC.len())?;
    if magic != MAGIC {
        return Err(SumcheckCodecError::InvalidMagic);
    }

    let modulus_bits = read_u64(input, &mut cursor)?;
    if modulus_bits != F::MODULUS_BIT_SIZE as u64 {
        return Err(SumcheckCodecError::InvalidFieldElement);
    }

    let rounds_u64 = read_u64(input, &mut cursor)?;
    let rounds: usize = rounds_u64
        .try_into()
        .map_err(|_| SumcheckCodecError::LengthOverflow)?;

    let mut round_polynomials = Vec::with_capacity(rounds);

    for _ in 0..rounds {
        round_polynomials.push(RoundPolynomial {
            evaluation_at_zero: read_field(input, &mut cursor)?,
            evaluation_at_one: read_field(input, &mut cursor)?,
        });
    }

    let final_evaluation = read_field(input, &mut cursor)?;

    if cursor != input.len() {
        return Err(SumcheckCodecError::TrailingBytes);
    }

    Ok(Proof {
        round_polynomials,
        final_evaluation,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use ark_bls12_381::Fr;

    fn proof_fixture() -> Proof<Fr> {
        Proof {
            round_polynomials: vec![
                RoundPolynomial {
                    evaluation_at_zero: Fr::from(3),
                    evaluation_at_one: Fr::from(5),
                },
                RoundPolynomial {
                    evaluation_at_zero: Fr::from(7),
                    evaluation_at_one: Fr::from(11),
                },
            ],
            final_evaluation: Fr::from(13),
        }
    }

    #[test]
    fn proof_roundtrip_is_canonical() {
        let proof = proof_fixture();

        let encoded = encode_proof(&proof).unwrap();
        let decoded = decode_proof::<Fr>(&encoded).unwrap();
        let reencoded = encode_proof(&decoded).unwrap();

        assert_eq!(proof, decoded);
        assert_eq!(encoded, reencoded);
    }

    #[test]
    fn rejects_wrong_magic() {
        let proof = proof_fixture();
        let mut encoded = encode_proof(&proof).unwrap();

        encoded[0] = b'X';

        assert_eq!(
            decode_proof::<Fr>(&encoded),
            Err(SumcheckCodecError::InvalidMagic)
        );
    }

    #[test]
    fn rejects_truncated_bytes() {
        let proof = proof_fixture();
        let mut encoded = encode_proof(&proof).unwrap();

        encoded.pop();

        assert_eq!(
            decode_proof::<Fr>(&encoded),
            Err(SumcheckCodecError::Truncated)
        );
    }

    #[test]
    fn rejects_trailing_bytes() {
        let proof = proof_fixture();
        let mut encoded = encode_proof(&proof).unwrap();

        encoded.push(0);

        assert_eq!(
            decode_proof::<Fr>(&encoded),
            Err(SumcheckCodecError::TrailingBytes)
        );
    }
}
