use ark_ff::PrimeField;
use snark_lab_transcript::ProofTranscript;

use crate::ipa::IpaCommitment;
use crate::pcs::{validate_opening_point, PcsShapeError};

const IPA_OPENING_STATEMENT_DOMAIN: &[u8] = b"snark-lab/ipa-opening-statement/v1";
const IPA_REDUCTION_ROUND_DOMAIN: &[u8] = b"snark-lab/ipa-reduction-round/v1";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IpaRoundSide {
    Left,
    Right,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IpaTranscriptError {
    Shape(PcsShapeError),
    EmptyRoundCommitment {
        round_index: usize,
        side: IpaRoundSide,
    },
    RoundCountMismatch {
        expected: usize,
        actual: usize,
    },
}

impl From<PcsShapeError> for IpaTranscriptError {
    fn from(error: PcsShapeError) -> Self {
        Self::Shape(error)
    }
}

/// One IPA reduction-round message.
///
/// In a real IPA PCS, the left and right commitments are group elements.
/// For now they remain canonical byte placeholders so the transcript shape is
/// fixed without pretending that cryptographic verification exists.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IpaTranscriptRound {
    pub round_index: usize,
    pub left_commitment_bytes: Vec<u8>,
    pub right_commitment_bytes: Vec<u8>,
}

impl IpaTranscriptRound {
    pub fn new(
        round_index: usize,
        left_commitment_bytes: Vec<u8>,
        right_commitment_bytes: Vec<u8>,
    ) -> Result<Self, IpaTranscriptError> {
        if left_commitment_bytes.is_empty() {
            return Err(IpaTranscriptError::EmptyRoundCommitment {
                round_index,
                side: IpaRoundSide::Left,
            });
        }

        if right_commitment_bytes.is_empty() {
            return Err(IpaTranscriptError::EmptyRoundCommitment {
                round_index,
                side: IpaRoundSide::Right,
            });
        }

        Ok(Self {
            round_index,
            left_commitment_bytes,
            right_commitment_bytes,
        })
    }
}

/// A multilinear IPA opening over `variables` variables has one reduction round
/// per variable.
pub fn expected_ipa_rounds(variables: usize) -> usize {
    variables
}

pub fn validate_ipa_round_count(
    variables: usize,
    actual_rounds: usize,
) -> Result<(), IpaTranscriptError> {
    let expected = expected_ipa_rounds(variables);

    if expected != actual_rounds {
        return Err(IpaTranscriptError::RoundCountMismatch {
            expected,
            actual: actual_rounds,
        });
    }

    Ok(())
}

/// Bind the opening statement before any IPA reduction rounds.
///
/// This binds:
///
/// - field modulus,
/// - variable count,
/// - commitment bytes,
/// - opening point.
pub fn bind_ipa_opening_statement<F: PrimeField, T: ProofTranscript<F>>(
    transcript: &mut T,
    commitment: &IpaCommitment,
    point: &[F],
) -> Result<(), IpaTranscriptError> {
    validate_opening_point(commitment.variables, point.len())?;

    transcript.append_domain_separator(IPA_OPENING_STATEMENT_DOMAIN);
    transcript.append_field_modulus();
    transcript.append_u64(b"ipa-statement-variables", commitment.variables as u64);
    transcript.append_bytes(b"ipa-statement-commitment", &commitment.commitment_bytes);
    transcript.append_u64(b"ipa-statement-point-len", point.len() as u64);

    for (index, coordinate) in point.iter().enumerate() {
        transcript.append_u64(b"ipa-statement-point-index", index as u64);
        transcript.append_field_element(b"ipa-statement-point-coordinate", coordinate);
    }

    Ok(())
}

/// Bind one IPA reduction round and derive its Fiat-Shamir challenge.
///
/// This does not verify any group relation yet. It only fixes the transcript
/// schedule used by the future IPA verifier.
pub fn absorb_ipa_reduction_round<F: PrimeField, T: ProofTranscript<F>>(
    transcript: &mut T,
    round: &IpaTranscriptRound,
) -> F {
    transcript.append_domain_separator(IPA_REDUCTION_ROUND_DOMAIN);
    transcript.append_u64(b"ipa-round-index", round.round_index as u64);
    transcript.append_bytes(b"ipa-round-left-commitment", &round.left_commitment_bytes);
    transcript.append_bytes(b"ipa-round-right-commitment", &round.right_commitment_bytes);

    transcript.challenge_scalar(b"ipa-round-challenge")
}

#[cfg(test)]
mod tests {
    use super::*;
    use ark_bls12_381::Fr;
    use snark_lab_transcript::{MerlinTranscript, ProofTranscript};

    fn round_challenge(left: &[u8], right: &[u8]) -> Fr {
        let round = IpaTranscriptRound::new(0, left.to_vec(), right.to_vec()).unwrap();
        let mut transcript = MerlinTranscript::new(b"ipa-transcript-test");

        absorb_ipa_reduction_round::<Fr, _>(&mut transcript, &round)
    }

    fn statement_challenge(point: &[Fr]) -> Result<Fr, IpaTranscriptError> {
        let commitment = IpaCommitment {
            variables: point.len(),
            commitment_bytes: vec![1, 2, 3],
        };

        let mut transcript = MerlinTranscript::new(b"ipa-statement-test");
        bind_ipa_opening_statement::<Fr, _>(&mut transcript, &commitment, point)?;

        Ok(transcript.challenge_scalar(b"after-ipa-statement"))
    }

    #[test]
    fn transcript_round_rejects_empty_left_commitment() {
        assert_eq!(
            IpaTranscriptRound::new(0, Vec::new(), vec![1]),
            Err(IpaTranscriptError::EmptyRoundCommitment {
                round_index: 0,
                side: IpaRoundSide::Left
            })
        );
    }

    #[test]
    fn transcript_round_rejects_empty_right_commitment() {
        assert_eq!(
            IpaTranscriptRound::new(2, vec![1], Vec::new()),
            Err(IpaTranscriptError::EmptyRoundCommitment {
                round_index: 2,
                side: IpaRoundSide::Right
            })
        );
    }

    #[test]
    fn ipa_round_challenge_is_deterministic() {
        assert_eq!(
            round_challenge(&[1, 2], &[3, 4]),
            round_challenge(&[1, 2], &[3, 4])
        );
    }

    #[test]
    fn ipa_round_challenge_changes_when_left_commitment_changes() {
        assert_ne!(
            round_challenge(&[1, 2], &[3, 4]),
            round_challenge(&[9, 2], &[3, 4])
        );
    }

    #[test]
    fn ipa_opening_statement_binds_point() {
        let a = statement_challenge(&[Fr::from(1), Fr::from(2), Fr::from(3)]).unwrap();
        let b = statement_challenge(&[Fr::from(1), Fr::from(2), Fr::from(4)]).unwrap();

        assert_ne!(a, b);
    }

    #[test]
    fn ipa_opening_statement_rejects_wrong_point_length() {
        let commitment = IpaCommitment {
            variables: 3,
            commitment_bytes: vec![1, 2, 3],
        };

        let mut transcript = MerlinTranscript::new(b"ipa-statement-test");

        assert_eq!(
            bind_ipa_opening_statement::<Fr, _>(&mut transcript, &commitment, &[Fr::from(1)]),
            Err(IpaTranscriptError::Shape(
                PcsShapeError::PointLengthMismatch {
                    expected: 3,
                    actual: 1
                }
            ))
        );
    }

    #[test]
    fn ipa_round_count_matches_variable_count() {
        assert_eq!(expected_ipa_rounds(0), 0);
        assert_eq!(expected_ipa_rounds(4), 4);
        assert_eq!(validate_ipa_round_count(4, 4), Ok(()));
    }

    #[test]
    fn ipa_round_count_rejects_wrong_count() {
        assert_eq!(
            validate_ipa_round_count(4, 3),
            Err(IpaTranscriptError::RoundCountMismatch {
                expected: 4,
                actual: 3
            })
        );
    }
}
