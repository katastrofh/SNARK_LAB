use ark_ec::{AffineRepr, CurveGroup};
use ark_ff::PrimeField;
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};

use crate::ipa_commitment::{IpaCommitmentEquationError, IpaCurveCommitment};
use crate::ipa_curve::{IpaCurveGeneratorBasis, IpaCurvePoint, IpaCurvePointError};
use crate::ipa_reduction::{
    validate_ipa_reduction_input_length, IpaReductionRound, IpaReductionRoundError,
};
use crate::ipa_transcript::{IpaTranscriptError, IpaTranscriptRound};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IpaRoundCommitmentError<F: PrimeField> {
    Curve(IpaCurvePointError),
    Commitment(IpaCommitmentEquationError),
    Transcript(IpaTranscriptError),
    Reduction(IpaReductionRoundError<F>),
    LengthMismatch {
        left: usize,
        right: usize,
    },
    GeneratorCountMismatch {
        label: &'static str,
        expected: usize,
        actual: usize,
    },
}

impl<F: PrimeField> From<IpaCurvePointError> for IpaRoundCommitmentError<F> {
    fn from(error: IpaCurvePointError) -> Self {
        Self::Curve(error)
    }
}

impl<F: PrimeField> From<IpaCommitmentEquationError> for IpaRoundCommitmentError<F> {
    fn from(error: IpaCommitmentEquationError) -> Self {
        Self::Commitment(error)
    }
}

impl<F: PrimeField> From<IpaTranscriptError> for IpaRoundCommitmentError<F> {
    fn from(error: IpaTranscriptError) -> Self {
        Self::Transcript(error)
    }
}

impl<F: PrimeField> From<IpaReductionRoundError<F>> for IpaRoundCommitmentError<F> {
    fn from(error: IpaReductionRoundError<F>) -> Self {
        Self::Reduction(error)
    }
}

/// Concrete IPA `L` and `R` round commitments.
///
/// These commitments use the standard inner-product reduction terms:
///
/// ```text
/// L = <a_L, G_R> + <b_R, H_L> + <a_L, b_R> U
/// R = <a_R, G_L> + <b_L, H_R> + <a_R, b_L> U
/// ```
///
/// This is a real algebraic component, not a verifier. The verifier loop still
/// has to consume these commitments, derive challenges, fold generators, and
/// check the final relation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IpaRoundCommitments<G: CurveGroup> {
    pub round_index: usize,
    pub left_commitment: IpaCurveCommitment<G>,
    pub right_commitment: IpaCurveCommitment<G>,
    pub left_cross_term: G::ScalarField,
    pub right_cross_term: G::ScalarField,
    pub input_length: usize,
}

impl<G> IpaRoundCommitments<G>
where
    G: CurveGroup,
    G::Affine: AffineRepr + CanonicalSerialize + CanonicalDeserialize,
{
    pub fn left_commitment_bytes(
        &self,
    ) -> Result<Vec<u8>, IpaRoundCommitmentError<G::ScalarField>> {
        Ok(self.left_commitment.to_compressed_bytes()?)
    }

    pub fn right_commitment_bytes(
        &self,
    ) -> Result<Vec<u8>, IpaRoundCommitmentError<G::ScalarField>> {
        Ok(self.right_commitment.to_compressed_bytes()?)
    }

    pub fn to_transcript_round(
        &self,
    ) -> Result<IpaTranscriptRound, IpaRoundCommitmentError<G::ScalarField>> {
        Ok(IpaTranscriptRound::new(
            self.round_index,
            self.left_commitment_bytes()?,
            self.right_commitment_bytes()?,
        )?)
    }

    pub fn to_reduction_round(
        &self,
        challenge: G::ScalarField,
    ) -> Result<IpaReductionRound<G::ScalarField>, IpaRoundCommitmentError<G::ScalarField>> {
        Ok(IpaReductionRound::new(
            self.round_index,
            self.left_commitment_bytes()?,
            self.right_commitment_bytes()?,
            challenge,
            self.input_length,
        )?)
    }
}

fn split_equal_halves<T>(values: &[T]) -> (&[T], &[T]) {
    let midpoint = values.len() / 2;
    values.split_at(midpoint)
}

fn inner_product<F: PrimeField>(left: &[F], right: &[F]) -> Result<F, IpaRoundCommitmentError<F>> {
    if left.len() != right.len() {
        return Err(IpaRoundCommitmentError::LengthMismatch {
            left: left.len(),
            right: right.len(),
        });
    }

    Ok(left
        .iter()
        .zip(right.iter())
        .map(|(left_value, right_value)| *left_value * *right_value)
        .sum())
}

fn linear_combination<G>(
    scalars: &[G::ScalarField],
    generators: &[IpaCurvePoint<G>],
) -> Result<G, IpaRoundCommitmentError<G::ScalarField>>
where
    G: CurveGroup,
    G::Affine: AffineRepr + CanonicalSerialize + CanonicalDeserialize,
{
    if scalars.len() != generators.len() {
        return Err(IpaRoundCommitmentError::LengthMismatch {
            left: scalars.len(),
            right: generators.len(),
        });
    }

    let mut accumulator = G::zero();

    for (scalar, generator) in scalars.iter().zip(generators.iter()) {
        accumulator += generator.affine().into_group() * *scalar;
    }

    Ok(accumulator)
}

fn validate_round_commitment_inputs<G>(
    basis: &IpaCurveGeneratorBasis<G>,
    polynomial_vector: &[G::ScalarField],
    evaluation_vector: &[G::ScalarField],
) -> Result<(), IpaRoundCommitmentError<G::ScalarField>>
where
    G: CurveGroup,
    G::Affine: AffineRepr + CanonicalSerialize + CanonicalDeserialize,
{
    validate_ipa_reduction_input_length::<G::ScalarField>(polynomial_vector.len())?;

    if polynomial_vector.len() != evaluation_vector.len() {
        return Err(IpaRoundCommitmentError::LengthMismatch {
            left: polynomial_vector.len(),
            right: evaluation_vector.len(),
        });
    }

    basis.validate()?;

    if basis.polynomial_generators.len() != polynomial_vector.len() {
        return Err(IpaRoundCommitmentError::GeneratorCountMismatch {
            label: "polynomial",
            expected: polynomial_vector.len(),
            actual: basis.polynomial_generators.len(),
        });
    }

    if basis.evaluation_generators.len() != evaluation_vector.len() {
        return Err(IpaRoundCommitmentError::GeneratorCountMismatch {
            label: "evaluation",
            expected: evaluation_vector.len(),
            actual: basis.evaluation_generators.len(),
        });
    }

    Ok(())
}

/// Compute one round's IPA `L` and `R` commitments.
pub fn compute_ipa_round_commitments<G>(
    round_index: usize,
    basis: &IpaCurveGeneratorBasis<G>,
    polynomial_vector: &[G::ScalarField],
    evaluation_vector: &[G::ScalarField],
    inner_product_generator: &IpaCurvePoint<G>,
) -> Result<IpaRoundCommitments<G>, IpaRoundCommitmentError<G::ScalarField>>
where
    G: CurveGroup,
    G::Affine: AffineRepr + CanonicalSerialize + CanonicalDeserialize,
{
    validate_round_commitment_inputs(basis, polynomial_vector, evaluation_vector)?;

    let (a_left, a_right) = split_equal_halves(polynomial_vector);
    let (b_left, b_right) = split_equal_halves(evaluation_vector);
    let (g_left, g_right) = split_equal_halves(&basis.polynomial_generators);
    let (h_left, h_right) = split_equal_halves(&basis.evaluation_generators);

    let left_cross_term = inner_product(a_left, b_right)?;
    let right_cross_term = inner_product(a_right, b_left)?;

    let mut left_commitment_group = linear_combination::<G>(a_left, g_right)?;
    left_commitment_group += linear_combination::<G>(b_right, h_left)?;
    left_commitment_group += inner_product_generator.affine().into_group() * left_cross_term;

    let mut right_commitment_group = linear_combination::<G>(a_right, g_left)?;
    right_commitment_group += linear_combination::<G>(b_left, h_right)?;
    right_commitment_group += inner_product_generator.affine().into_group() * right_cross_term;

    Ok(IpaRoundCommitments {
        round_index,
        left_commitment: IpaCurveCommitment::from_projective(left_commitment_group)?,
        right_commitment: IpaCurveCommitment::from_projective(right_commitment_group)?,
        left_cross_term,
        right_cross_term,
        input_length: polynomial_vector.len(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use ark_bls12_381::{Fr, G1Projective};
    use ark_ec::{AffineRepr, PrimeGroup};
    use ark_ff::Zero;

    use crate::ipa_curve::{IpaCurveGeneratorBasis, IpaCurvePoint};
    use crate::ipa_generators::expected_ipa_generator_count;

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

    fn scalars(values: &[u64]) -> Vec<Fr> {
        values.iter().copied().map(Fr::from).collect()
    }

    fn commitment_group(commitment: &IpaCurveCommitment<G1Projective>) -> G1Projective {
        commitment.affine().into_group()
    }

    #[test]
    fn round_commitments_match_manual_formula() {
        let basis = basis(2);
        let a = scalars(&[2, 3, 5, 7]);
        let b = scalars(&[11, 13, 17, 19]);
        let u = point(5000);

        let commitments = compute_ipa_round_commitments(0, &basis, &a, &b, &u).unwrap();

        let left_cross = a[0] * b[2] + a[1] * b[3];
        let right_cross = a[2] * b[0] + a[3] * b[1];

        let mut manual_l = G1Projective::zero();
        manual_l += basis.polynomial_generators[2].affine().into_group() * a[0];
        manual_l += basis.polynomial_generators[3].affine().into_group() * a[1];
        manual_l += basis.evaluation_generators[0].affine().into_group() * b[2];
        manual_l += basis.evaluation_generators[1].affine().into_group() * b[3];
        manual_l += u.affine().into_group() * left_cross;

        let mut manual_r = G1Projective::zero();
        manual_r += basis.polynomial_generators[0].affine().into_group() * a[2];
        manual_r += basis.polynomial_generators[1].affine().into_group() * a[3];
        manual_r += basis.evaluation_generators[2].affine().into_group() * b[0];
        manual_r += basis.evaluation_generators[3].affine().into_group() * b[1];
        manual_r += u.affine().into_group() * right_cross;

        assert_eq!(commitment_group(&commitments.left_commitment), manual_l);
        assert_eq!(commitment_group(&commitments.right_commitment), manual_r);
        assert_eq!(commitments.left_cross_term, left_cross);
        assert_eq!(commitments.right_cross_term, right_cross);
    }

    #[test]
    fn round_commitments_produce_transcript_round() {
        let basis = basis(2);
        let a = scalars(&[2, 3, 5, 7]);
        let b = scalars(&[11, 13, 17, 19]);
        let u = point(5000);

        let commitments = compute_ipa_round_commitments(3, &basis, &a, &b, &u).unwrap();
        let transcript_round = commitments.to_transcript_round().unwrap();

        assert_eq!(transcript_round.round_index, 3);
        assert!(!transcript_round.left_commitment_bytes.is_empty());
        assert!(!transcript_round.right_commitment_bytes.is_empty());
    }

    #[test]
    fn round_commitments_produce_reduction_round() {
        let basis = basis(2);
        let a = scalars(&[2, 3, 5, 7]);
        let b = scalars(&[11, 13, 17, 19]);
        let u = point(5000);

        let commitments = compute_ipa_round_commitments(0, &basis, &a, &b, &u).unwrap();
        let reduction_round = commitments.to_reduction_round(Fr::from(7)).unwrap();

        assert_eq!(reduction_round.input_length, 4);
        assert_eq!(reduction_round.output_length, 2);
        assert_eq!(reduction_round.round_index(), 0);
    }

    #[test]
    fn round_commitments_are_deterministic() {
        let basis = basis(2);
        let a = scalars(&[2, 3, 5, 7]);
        let b = scalars(&[11, 13, 17, 19]);
        let u = point(5000);

        let first = compute_ipa_round_commitments(0, &basis, &a, &b, &u).unwrap();
        let second = compute_ipa_round_commitments(0, &basis, &a, &b, &u).unwrap();

        assert_eq!(first, second);
    }

    #[test]
    fn round_commitments_change_when_inner_product_generator_changes() {
        let basis = basis(2);
        let a = scalars(&[2, 3, 5, 7]);
        let b = scalars(&[11, 13, 17, 19]);

        let first = compute_ipa_round_commitments(0, &basis, &a, &b, &point(5000)).unwrap();
        let second = compute_ipa_round_commitments(0, &basis, &a, &b, &point(5001)).unwrap();

        assert_ne!(first, second);
    }

    #[test]
    fn round_commitments_reject_vector_length_mismatch() {
        let basis = basis(2);
        let a = scalars(&[2, 3, 5, 7]);
        let b = scalars(&[11, 13]);
        let u = point(5000);

        assert_eq!(
            compute_ipa_round_commitments(0, &basis, &a, &b, &u),
            Err(IpaRoundCommitmentError::LengthMismatch { left: 4, right: 2 })
        );
    }

    #[test]
    fn round_commitments_reject_odd_length() {
        let basis = basis(2);
        let a = scalars(&[2, 3, 5]);
        let b = scalars(&[11, 13, 17]);
        let u = point(5000);

        assert!(matches!(
            compute_ipa_round_commitments(0, &basis, &a, &b, &u),
            Err(IpaRoundCommitmentError::Reduction(
                IpaReductionRoundError::OddInputLength { actual: 3 }
            ))
        ));
    }

    #[test]
    fn round_commitments_reject_bad_generator_count() {
        let mut basis = basis(2);
        basis.polynomial_generators.pop();

        let a = scalars(&[2, 3, 5, 7]);
        let b = scalars(&[11, 13, 17, 19]);
        let u = point(5000);

        assert!(matches!(
            compute_ipa_round_commitments(0, &basis, &a, &b, &u),
            Err(IpaRoundCommitmentError::Curve(
                IpaCurvePointError::InvalidGeneratorCount { .. }
            ))
        ));
    }
}
