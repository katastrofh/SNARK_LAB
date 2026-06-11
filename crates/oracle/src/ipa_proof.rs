use ark_ff::PrimeField;

use crate::ipa_transcript::{
    expected_ipa_rounds, validate_ipa_round_count, IpaTranscriptError, IpaTranscriptRound,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IpaProofShapeError {
    Transcript(IpaTranscriptError),
    EmptyFinalCommitment,
}

impl From<IpaTranscriptError> for IpaProofShapeError {
    fn from(error: IpaTranscriptError) -> Self {
        Self::Transcript(error)
    }
}

/// Typed IPA opening proof shape.
///
/// This is not yet a complete cryptographic proof implementation.
///
/// It fixes the public proof object layout that a real IPA backend will later
/// populate with group-element commitments and folded scalar claims.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IpaOpeningProof<F: PrimeField> {
    pub variables: usize,
    pub claimed_value: F,
    pub rounds: Vec<IpaTranscriptRound>,
    pub final_polynomial_scalar: F,
    pub final_evaluation_basis_scalar: F,
    pub final_commitment_bytes: Vec<u8>,
}

impl<F: PrimeField> IpaOpeningProof<F> {
    pub fn new(
        variables: usize,
        claimed_value: F,
        rounds: Vec<IpaTranscriptRound>,
        final_polynomial_scalar: F,
        final_evaluation_basis_scalar: F,
        final_commitment_bytes: Vec<u8>,
    ) -> Result<Self, IpaProofShapeError> {
        validate_ipa_round_count(variables, rounds.len())?;

        if final_commitment_bytes.is_empty() {
            return Err(IpaProofShapeError::EmptyFinalCommitment);
        }

        Ok(Self {
            variables,
            claimed_value,
            rounds,
            final_polynomial_scalar,
            final_evaluation_basis_scalar,
            final_commitment_bytes,
        })
    }

    pub fn expected_rounds(&self) -> usize {
        expected_ipa_rounds(self.variables)
    }
}

pub fn validate_ipa_opening_proof_shape<F: PrimeField>(
    proof: &IpaOpeningProof<F>,
) -> Result<(), IpaProofShapeError> {
    validate_ipa_round_count(proof.variables, proof.rounds.len())?;

    if proof.final_commitment_bytes.is_empty() {
        return Err(IpaProofShapeError::EmptyFinalCommitment);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ark_bls12_381::Fr;

    fn round(index: usize) -> IpaTranscriptRound {
        IpaTranscriptRound::new(index, vec![index as u8 + 1], vec![index as u8 + 2]).unwrap()
    }

    fn proof_with_rounds(
        rounds: Vec<IpaTranscriptRound>,
    ) -> Result<IpaOpeningProof<Fr>, IpaProofShapeError> {
        IpaOpeningProof::new(
            3,
            Fr::from(9),
            rounds,
            Fr::from(7),
            Fr::from(8),
            vec![1, 2, 3],
        )
    }

    #[test]
    fn opening_proof_shape_accepts_expected_round_count() {
        let proof = proof_with_rounds(vec![round(0), round(1), round(2)]).unwrap();

        assert_eq!(proof.variables, 3);
        assert_eq!(proof.expected_rounds(), 3);
        assert_eq!(validate_ipa_opening_proof_shape(&proof), Ok(()));
    }

    #[test]
    fn opening_proof_shape_rejects_wrong_round_count() {
        assert_eq!(
            proof_with_rounds(vec![round(0), round(1)]),
            Err(IpaProofShapeError::Transcript(
                IpaTranscriptError::RoundCountMismatch {
                    expected: 3,
                    actual: 2
                }
            ))
        );
    }

    #[test]
    fn opening_proof_shape_rejects_empty_final_commitment() {
        assert_eq!(
            IpaOpeningProof::new(
                1,
                Fr::from(9),
                vec![round(0)],
                Fr::from(7),
                Fr::from(8),
                Vec::new(),
            ),
            Err(IpaProofShapeError::EmptyFinalCommitment)
        );
    }

    #[test]
    fn zero_variable_proof_has_zero_rounds() {
        let proof = IpaOpeningProof::new(
            0,
            Fr::from(9),
            Vec::new(),
            Fr::from(7),
            Fr::from(8),
            vec![1],
        )
        .unwrap();

        assert_eq!(proof.expected_rounds(), 0);
        assert_eq!(validate_ipa_opening_proof_shape(&proof), Ok(()));
    }
}
