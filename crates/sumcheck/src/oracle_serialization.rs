use ark_ff::PrimeField;
use snark_lab_oracle::{TransparentCommitment, TransparentOpening, TransparentOracle};

use crate::{decode_proof, encode_proof, OracleProof, SumcheckCodecError};

const MAGIC: &[u8] = b"SL-ORACLE-SUM1";

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OracleProofCodecError {
    InvalidMagic,
    Truncated,
    TrailingBytes,
    InvalidFieldElement,
    LengthOverflow,
    InnerSumcheck(SumcheckCodecError),
}

fn push_u64(out: &mut Vec<u8>, value: u64) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn read_exact<'a>(
    input: &'a [u8],
    cursor: &mut usize,
    len: usize,
) -> Result<&'a [u8], OracleProofCodecError> {
    let end = cursor
        .checked_add(len)
        .ok_or(OracleProofCodecError::LengthOverflow)?;

    if end > input.len() {
        return Err(OracleProofCodecError::Truncated);
    }

    let bytes = &input[*cursor..end];
    *cursor = end;
    Ok(bytes)
}

fn read_u64(input: &[u8], cursor: &mut usize) -> Result<u64, OracleProofCodecError> {
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
) -> Result<F, OracleProofCodecError> {
    let len = field_bytes_len::<F>();
    let bytes = read_exact(input, cursor, len)?;

    let decoded = F::from_le_bytes_mod_order(bytes);

    let mut canonical = Vec::with_capacity(len);
    write_field(&mut canonical, &decoded);

    if canonical.as_slice() != bytes {
        return Err(OracleProofCodecError::InvalidFieldElement);
    }

    Ok(decoded)
}

pub fn encode_transparent_oracle_proof<F: PrimeField>(
    proof: &OracleProof<F, TransparentOracle<F>>,
) -> Result<Vec<u8>, OracleProofCodecError> {
    let variables: u64 = proof
        .commitment
        .variables
        .try_into()
        .map_err(|_| OracleProofCodecError::LengthOverflow)?;

    let evaluations_len: u64 = proof
        .commitment
        .evaluations
        .len()
        .try_into()
        .map_err(|_| OracleProofCodecError::LengthOverflow)?;

    let inner_sumcheck = encode_proof(&proof.sumcheck_proof)
        .map_err(OracleProofCodecError::InnerSumcheck)?;

    let inner_len: u64 = inner_sumcheck
        .len()
        .try_into()
        .map_err(|_| OracleProofCodecError::LengthOverflow)?;

    let mut out = Vec::new();

    out.extend_from_slice(MAGIC);
    push_u64(&mut out, F::MODULUS_BIT_SIZE as u64);

    push_u64(&mut out, variables);
    push_u64(&mut out, evaluations_len);

    for evaluation in &proof.commitment.evaluations {
        write_field(&mut out, evaluation);
    }

    push_u64(&mut out, inner_len);
    out.extend_from_slice(&inner_sumcheck);

    write_field(&mut out, &proof.final_opening.value);

    Ok(out)
}

pub fn decode_transparent_oracle_proof<F: PrimeField>(
    input: &[u8],
) -> Result<OracleProof<F, TransparentOracle<F>>, OracleProofCodecError> {
    let mut cursor = 0usize;

    let magic = read_exact(input, &mut cursor, MAGIC.len())?;
    if magic != MAGIC {
        return Err(OracleProofCodecError::InvalidMagic);
    }

    let modulus_bits = read_u64(input, &mut cursor)?;
    if modulus_bits != F::MODULUS_BIT_SIZE as u64 {
        return Err(OracleProofCodecError::InvalidFieldElement);
    }

    let variables_u64 = read_u64(input, &mut cursor)?;
    let variables: usize = variables_u64
        .try_into()
        .map_err(|_| OracleProofCodecError::LengthOverflow)?;

    let evaluations_len_u64 = read_u64(input, &mut cursor)?;
    let evaluations_len: usize = evaluations_len_u64
        .try_into()
        .map_err(|_| OracleProofCodecError::LengthOverflow)?;

    let mut evaluations = Vec::with_capacity(evaluations_len);
    for _ in 0..evaluations_len {
        evaluations.push(read_field(input, &mut cursor)?);
    }

    let inner_len_u64 = read_u64(input, &mut cursor)?;
    let inner_len: usize = inner_len_u64
        .try_into()
        .map_err(|_| OracleProofCodecError::LengthOverflow)?;

    let inner_bytes = read_exact(input, &mut cursor, inner_len)?;
    let sumcheck_proof =
        decode_proof::<F>(inner_bytes).map_err(OracleProofCodecError::InnerSumcheck)?;

    let final_opening = TransparentOpening {
        value: read_field(input, &mut cursor)?,
    };

    if cursor != input.len() {
        return Err(OracleProofCodecError::TrailingBytes);
    }

    Ok(OracleProof {
        commitment: TransparentCommitment {
            variables,
            evaluations,
        },
        sumcheck_proof,
        final_opening,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use ark_bls12_381::Fr;
    use multilinear::Multilinear;
    use snark_lab_oracle::MultilinearOracle;
    use snark_lab_transcript::MerlinTranscript;

    use crate::{prove_with_transparent_oracle, verify_with_oracle};

    fn fixture_proof() -> (Fr, OracleProof<Fr, TransparentOracle<Fr>>) {
        let polynomial = Multilinear::new((0u64..8).map(Fr::from).collect()).unwrap();
        let oracle = TransparentOracle::new(polynomial);
        let claim = oracle.sum_hypercube();

        let mut transcript = MerlinTranscript::new(b"transparent-oracle-codec-test");
        let proof = prove_with_transparent_oracle(&oracle, claim, &mut transcript).unwrap();

        (claim, proof)
    }

    #[test]
    fn transparent_oracle_proof_roundtrip_is_canonical() {
        let (_claim, proof) = fixture_proof();

        let encoded = encode_transparent_oracle_proof(&proof).unwrap();
        let decoded = decode_transparent_oracle_proof::<Fr>(&encoded).unwrap();
        let reencoded = encode_transparent_oracle_proof(&decoded).unwrap();

        assert_eq!(proof, decoded);
        assert_eq!(encoded, reencoded);
    }

    #[test]
    fn decoded_transparent_oracle_proof_verifies() {
        let (claim, proof) = fixture_proof();

        let encoded = encode_transparent_oracle_proof(&proof).unwrap();
        let decoded = decode_transparent_oracle_proof::<Fr>(&encoded).unwrap();

        let mut transcript = MerlinTranscript::new(b"transparent-oracle-codec-test");
        let challenges = verify_with_oracle::<Fr, _, TransparentOracle<Fr>>(
            &decoded.commitment,
            decoded.commitment.variables,
            claim,
            &decoded.sumcheck_proof,
            &decoded.final_opening,
            &mut transcript,
        )
        .unwrap();

        assert_eq!(challenges.len(), decoded.commitment.variables);
    }

    #[test]
    fn rejects_wrong_magic() {
        let (_claim, proof) = fixture_proof();
        let mut encoded = encode_transparent_oracle_proof(&proof).unwrap();

        encoded[0] = b'X';

        assert_eq!(
            decode_transparent_oracle_proof::<Fr>(&encoded),
            Err(OracleProofCodecError::InvalidMagic)
        );
    }

    #[test]
    fn rejects_truncated_transparent_oracle_proof() {
        let (_claim, proof) = fixture_proof();
        let mut encoded = encode_transparent_oracle_proof(&proof).unwrap();

        encoded.pop();

        assert_eq!(
            decode_transparent_oracle_proof::<Fr>(&encoded),
            Err(OracleProofCodecError::Truncated)
        );
    }

    #[test]
    fn rejects_trailing_bytes() {
        let (_claim, proof) = fixture_proof();
        let mut encoded = encode_transparent_oracle_proof(&proof).unwrap();

        encoded.push(0);

        assert_eq!(
            decode_transparent_oracle_proof::<Fr>(&encoded),
            Err(OracleProofCodecError::TrailingBytes)
        );
    }
}
