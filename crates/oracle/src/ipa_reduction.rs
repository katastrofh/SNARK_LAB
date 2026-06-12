use ark_ff::PrimeField;
use snark_lab_transcript::ProofTranscript;

use crate::ipa_transcript::{IpaTranscriptError, IpaTranscriptRound};

const IPA_REDUCTION_ROUND_DOMAIN: &[u8] = b"snark-lab/ipa-reduction-round/v1";

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IpaReductionRoundError<F: PrimeField> {
    Transcript(IpaTranscriptError),
    InvalidInputLength { minimum: usize, actual: usize },
    OddInputLength { actual: usize },
    LengthMismatch { left: usize, right: usize },
    ZeroChallenge,
    FoldedLengthMismatch { expected: usize, actual: usize },
    UnexpectedChallenge { expected: F, actual: F },
}

impl<F: PrimeField> From<IpaTranscriptError> for IpaReductionRoundError<F> {
    fn from(error: IpaTranscriptError) -> Self {
        Self::Transcript(error)
    }
}

/// One checked IPA reduction round.
///
/// This stores the public round messages `L` and `R`, the Fiat-Shamir
/// challenge `x`, its inverse, and the vector dimensions before and after
/// folding.
///
/// It does not verify a full opening proof by itself. It is the algebraic round
/// object consumed by the future prover and verifier loops.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IpaReductionRound<F: PrimeField> {
    pub transcript_round: IpaTranscriptRound,
    pub challenge: F,
    pub challenge_inverse: F,
    pub input_length: usize,
    pub output_length: usize,
}

impl<F: PrimeField> IpaReductionRound<F> {
    pub fn new(
        round_index: usize,
        left_commitment_bytes: Vec<u8>,
        right_commitment_bytes: Vec<u8>,
        challenge: F,
        input_length: usize,
    ) -> Result<Self, IpaReductionRoundError<F>> {
        validate_ipa_reduction_input_length::<F>(input_length)?;

        let transcript_round =
            IpaTranscriptRound::new(round_index, left_commitment_bytes, right_commitment_bytes)?;

        let challenge_inverse = challenge
            .inverse()
            .ok_or(IpaReductionRoundError::ZeroChallenge)?;

        Ok(Self {
            transcript_round,
            challenge,
            challenge_inverse,
            input_length,
            output_length: input_length / 2,
        })
    }

    pub fn round_index(&self) -> usize {
        self.transcript_round.round_index
    }
}

pub fn validate_ipa_reduction_input_length<F: PrimeField>(
    input_length: usize,
) -> Result<(), IpaReductionRoundError<F>> {
    if input_length < 2 {
        return Err(IpaReductionRoundError::InvalidInputLength {
            minimum: 2,
            actual: input_length,
        });
    }

    if !input_length.is_multiple_of(2) {
        return Err(IpaReductionRoundError::OddInputLength {
            actual: input_length,
        });
    }

    Ok(())
}

fn split_equal_halves<F: PrimeField>(
    values: &[F],
) -> Result<(&[F], &[F]), IpaReductionRoundError<F>> {
    validate_ipa_reduction_input_length::<F>(values.len())?;

    let midpoint = values.len() / 2;
    let (left, right) = values.split_at(midpoint);

    if left.len() != right.len() {
        return Err(IpaReductionRoundError::LengthMismatch {
            left: left.len(),
            right: right.len(),
        });
    }

    Ok((left, right))
}

/// Fold the committed polynomial vector:
///
/// ```text
/// a' = x·a_L + x^{-1}·a_R
/// ```
pub fn fold_ipa_polynomial_vector<F: PrimeField>(
    values: &[F],
    challenge: F,
) -> Result<Vec<F>, IpaReductionRoundError<F>> {
    let challenge_inverse = challenge
        .inverse()
        .ok_or(IpaReductionRoundError::ZeroChallenge)?;

    let (left, right) = split_equal_halves(values)?;

    Ok(left
        .iter()
        .zip(right.iter())
        .map(|(left_value, right_value)| challenge * *left_value + challenge_inverse * *right_value)
        .collect())
}

/// Fold the evaluation-basis vector:
///
/// ```text
/// b' = x^{-1}·b_L + x·b_R
/// ```
pub fn fold_ipa_evaluation_vector<F: PrimeField>(
    values: &[F],
    challenge: F,
) -> Result<Vec<F>, IpaReductionRoundError<F>> {
    let challenge_inverse = challenge
        .inverse()
        .ok_or(IpaReductionRoundError::ZeroChallenge)?;

    let (left, right) = split_equal_halves(values)?;

    Ok(left
        .iter()
        .zip(right.iter())
        .map(|(left_value, right_value)| challenge_inverse * *left_value + challenge * *right_value)
        .collect())
}

/// Check that supplied folded vectors match the IPA folding rules.
pub fn validate_ipa_vector_fold<F: PrimeField>(
    original_polynomial_vector: &[F],
    original_evaluation_vector: &[F],
    folded_polynomial_vector: &[F],
    folded_evaluation_vector: &[F],
    challenge: F,
) -> Result<(), IpaReductionRoundError<F>> {
    if original_polynomial_vector.len() != original_evaluation_vector.len() {
        return Err(IpaReductionRoundError::LengthMismatch {
            left: original_polynomial_vector.len(),
            right: original_evaluation_vector.len(),
        });
    }

    let expected_polynomial = fold_ipa_polynomial_vector(original_polynomial_vector, challenge)?;
    let expected_evaluation = fold_ipa_evaluation_vector(original_evaluation_vector, challenge)?;

    if expected_polynomial.len() != folded_polynomial_vector.len() {
        return Err(IpaReductionRoundError::FoldedLengthMismatch {
            expected: expected_polynomial.len(),
            actual: folded_polynomial_vector.len(),
        });
    }

    if expected_evaluation.len() != folded_evaluation_vector.len() {
        return Err(IpaReductionRoundError::FoldedLengthMismatch {
            expected: expected_evaluation.len(),
            actual: folded_evaluation_vector.len(),
        });
    }

    if expected_polynomial != folded_polynomial_vector {
        return Err(IpaReductionRoundError::FoldedLengthMismatch {
            expected: expected_polynomial.len(),
            actual: folded_polynomial_vector.len(),
        });
    }

    if expected_evaluation != folded_evaluation_vector {
        return Err(IpaReductionRoundError::FoldedLengthMismatch {
            expected: expected_evaluation.len(),
            actual: folded_evaluation_vector.len(),
        });
    }

    Ok(())
}

/// Bind a checked reduction round into the Fiat-Shamir transcript.
///
/// In the final verifier this round object must be constructed from challenges
/// derived after absorbing `L` and `R`. This function only binds a checked round
/// state; it does not claim proof acceptance.
pub fn bind_ipa_reduction_round_context<F: PrimeField, T: ProofTranscript<F>>(
    transcript: &mut T,
    round: &IpaReductionRound<F>,
) -> Result<(), IpaReductionRoundError<F>> {
    validate_ipa_reduction_input_length::<F>(round.input_length)?;

    if round.output_length != round.input_length / 2 {
        return Err(IpaReductionRoundError::FoldedLengthMismatch {
            expected: round.input_length / 2,
            actual: round.output_length,
        });
    }

    let recomputed_inverse = round
        .challenge
        .inverse()
        .ok_or(IpaReductionRoundError::ZeroChallenge)?;

    if recomputed_inverse != round.challenge_inverse {
        return Err(IpaReductionRoundError::UnexpectedChallenge {
            expected: recomputed_inverse,
            actual: round.challenge_inverse,
        });
    }

    transcript.append_domain_separator(IPA_REDUCTION_ROUND_DOMAIN);
    transcript.append_field_modulus();
    transcript.append_u64(b"ipa-reduction-round-index", round.round_index() as u64);
    transcript.append_u64(b"ipa-reduction-input-length", round.input_length as u64);
    transcript.append_u64(b"ipa-reduction-output-length", round.output_length as u64);
    transcript.append_bytes(
        b"ipa-reduction-left-commitment",
        &round.transcript_round.left_commitment_bytes,
    );
    transcript.append_bytes(
        b"ipa-reduction-right-commitment",
        &round.transcript_round.right_commitment_bytes,
    );
    transcript.append_field_element(b"ipa-reduction-challenge", &round.challenge);
    transcript.append_field_element(b"ipa-reduction-challenge-inverse", &round.challenge_inverse);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ark_bls12_381::Fr;
    use ark_ff::Field;
    use snark_lab_transcript::{MerlinTranscript, ProofTranscript};

    fn values(values: &[u64]) -> Vec<Fr> {
        values.iter().copied().map(Fr::from).collect()
    }

    fn challenge() -> Fr {
        Fr::from(7)
    }

    fn round() -> IpaReductionRound<Fr> {
        IpaReductionRound::new(0, vec![1, 2, 3], vec![4, 5, 6], challenge(), 4).unwrap()
    }

    fn challenge_for_round(round: &IpaReductionRound<Fr>) -> Fr {
        let mut transcript = MerlinTranscript::new(b"ipa-reduction-round-test");

        bind_ipa_reduction_round_context::<Fr, _>(&mut transcript, round).unwrap();

        transcript.challenge_scalar(b"after-reduction-round")
    }

    #[test]
    fn reduction_round_accepts_valid_shape() {
        let round = round();

        assert_eq!(round.round_index(), 0);
        assert_eq!(round.input_length, 4);
        assert_eq!(round.output_length, 2);
        assert_eq!(round.challenge * round.challenge_inverse, Fr::from(1));
    }

    #[test]
    fn reduction_round_rejects_zero_challenge() {
        assert_eq!(
            IpaReductionRound::<Fr>::new(0, vec![1], vec![2], Fr::from(0), 4),
            Err(IpaReductionRoundError::ZeroChallenge)
        );
    }

    #[test]
    fn reduction_round_rejects_empty_left_commitment() {
        assert!(matches!(
            IpaReductionRound::<Fr>::new(0, Vec::new(), vec![2], challenge(), 4),
            Err(IpaReductionRoundError::Transcript(_))
        ));
    }

    #[test]
    fn reduction_round_rejects_too_short_vector() {
        assert_eq!(
            IpaReductionRound::<Fr>::new(0, vec![1], vec![2], challenge(), 1),
            Err(IpaReductionRoundError::InvalidInputLength {
                minimum: 2,
                actual: 1
            })
        );
    }

    #[test]
    fn reduction_round_rejects_odd_vector_length() {
        assert_eq!(
            IpaReductionRound::<Fr>::new(0, vec![1], vec![2], challenge(), 3),
            Err(IpaReductionRoundError::OddInputLength { actual: 3 })
        );
    }

    #[test]
    fn polynomial_vector_fold_matches_formula() {
        let input = values(&[2, 3, 5, 7]);
        let x = challenge();
        let x_inv = x.inverse().unwrap();

        let folded = fold_ipa_polynomial_vector(&input, x).unwrap();

        assert_eq!(
            folded,
            vec![
                x * input[0] + x_inv * input[2],
                x * input[1] + x_inv * input[3]
            ]
        );
    }

    #[test]
    fn evaluation_vector_fold_matches_formula() {
        let input = values(&[2, 3, 5, 7]);
        let x = challenge();
        let x_inv = x.inverse().unwrap();

        let folded = fold_ipa_evaluation_vector(&input, x).unwrap();

        assert_eq!(
            folded,
            vec![
                x_inv * input[0] + x * input[2],
                x_inv * input[1] + x * input[3]
            ]
        );
    }

    #[test]
    fn vector_fold_validator_accepts_matching_fold() {
        let a = values(&[2, 3, 5, 7]);
        let b = values(&[11, 13, 17, 19]);
        let x = challenge();

        let a_folded = fold_ipa_polynomial_vector(&a, x).unwrap();
        let b_folded = fold_ipa_evaluation_vector(&b, x).unwrap();

        assert_eq!(
            validate_ipa_vector_fold(&a, &b, &a_folded, &b_folded, x),
            Ok(())
        );
    }

    #[test]
    fn vector_fold_validator_rejects_mismatched_inputs() {
        let a = values(&[2, 3, 5, 7]);
        let b = values(&[11, 13]);

        assert_eq!(
            validate_ipa_vector_fold(&a, &b, &[], &[], challenge()),
            Err(IpaReductionRoundError::LengthMismatch { left: 4, right: 2 })
        );
    }

    #[test]
    fn reduction_round_binding_is_deterministic() {
        let round = round();

        assert_eq!(challenge_for_round(&round), challenge_for_round(&round));
    }

    #[test]
    fn reduction_round_binding_changes_when_left_commitment_changes() {
        let a = round();
        let b = IpaReductionRound::new(0, vec![9, 2, 3], vec![4, 5, 6], challenge(), 4).unwrap();

        assert_ne!(challenge_for_round(&a), challenge_for_round(&b));
    }

    #[test]
    fn reduction_round_binding_rejects_bad_inverse() {
        let mut round = round();
        round.challenge_inverse = Fr::from(123);

        assert!(matches!(
            bind_ipa_reduction_round_context(&mut MerlinTranscript::new(b"bad"), &round),
            Err(IpaReductionRoundError::UnexpectedChallenge { .. })
        ));
    }
}
