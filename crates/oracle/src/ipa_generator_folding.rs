use ark_ec::{AffineRepr, CurveGroup};
use ark_ff::{Field, PrimeField};
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};

use crate::ipa_curve::{IpaCurveGeneratorBasis, IpaCurvePoint, IpaCurvePointError};
use crate::ipa_reduction::{validate_ipa_reduction_input_length, IpaReductionRoundError};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IpaGeneratorFoldingError<F: PrimeField> {
    Curve(IpaCurvePointError),
    Reduction(IpaReductionRoundError<F>),
    LengthMismatch { left: usize, right: usize },
    ZeroChallenge,
    VariableUnderflow,
}

impl<F: PrimeField> From<IpaCurvePointError> for IpaGeneratorFoldingError<F> {
    fn from(error: IpaCurvePointError) -> Self {
        Self::Curve(error)
    }
}

impl<F: PrimeField> From<IpaReductionRoundError<F>> for IpaGeneratorFoldingError<F> {
    fn from(error: IpaReductionRoundError<F>) -> Self {
        Self::Reduction(error)
    }
}

fn split_equal_halves<T, F: PrimeField>(
    values: &[T],
) -> Result<(&[T], &[T]), IpaGeneratorFoldingError<F>> {
    validate_ipa_reduction_input_length::<F>(values.len())?;

    let midpoint = values.len() / 2;
    let (left, right) = values.split_at(midpoint);

    if left.len() != right.len() {
        return Err(IpaGeneratorFoldingError::LengthMismatch {
            left: left.len(),
            right: right.len(),
        });
    }

    Ok((left, right))
}

/// Fold polynomial commitment generators:
///
/// ```text
/// G' = x^{-1} G_L + x G_R
/// ```
pub fn fold_ipa_polynomial_generators<G>(
    generators: &[IpaCurvePoint<G>],
    challenge: G::ScalarField,
) -> Result<Vec<IpaCurvePoint<G>>, IpaGeneratorFoldingError<G::ScalarField>>
where
    G: CurveGroup,
    G::Affine: AffineRepr + CanonicalSerialize + CanonicalDeserialize,
{
    let challenge_inverse = challenge
        .inverse()
        .ok_or(IpaGeneratorFoldingError::ZeroChallenge)?;

    let (left, right) = split_equal_halves::<_, G::ScalarField>(generators)?;

    left.iter()
        .zip(right.iter())
        .map(|(left_generator, right_generator)| {
            let folded = left_generator.affine().into_group() * challenge_inverse
                + right_generator.affine().into_group() * challenge;

            Ok(IpaCurvePoint::from_projective(folded)?)
        })
        .collect()
}

/// Fold evaluation generators:
///
/// ```text
/// H' = x H_L + x^{-1} H_R
/// ```
pub fn fold_ipa_evaluation_generators<G>(
    generators: &[IpaCurvePoint<G>],
    challenge: G::ScalarField,
) -> Result<Vec<IpaCurvePoint<G>>, IpaGeneratorFoldingError<G::ScalarField>>
where
    G: CurveGroup,
    G::Affine: AffineRepr + CanonicalSerialize + CanonicalDeserialize,
{
    let challenge_inverse = challenge
        .inverse()
        .ok_or(IpaGeneratorFoldingError::ZeroChallenge)?;

    let (left, right) = split_equal_halves::<_, G::ScalarField>(generators)?;

    left.iter()
        .zip(right.iter())
        .map(|(left_generator, right_generator)| {
            let folded = left_generator.affine().into_group() * challenge
                + right_generator.affine().into_group() * challenge_inverse;

            Ok(IpaCurvePoint::from_projective(folded)?)
        })
        .collect()
}

/// Fold the full IPA generator basis by one reduction round.
///
/// The hiding/blinding generator is carried forward unchanged. It is not used
/// as the IPA inner-product generator.
pub fn fold_ipa_generator_basis<G>(
    basis: &IpaCurveGeneratorBasis<G>,
    challenge: G::ScalarField,
) -> Result<IpaCurveGeneratorBasis<G>, IpaGeneratorFoldingError<G::ScalarField>>
where
    G: CurveGroup,
    G::Affine: AffineRepr + CanonicalSerialize + CanonicalDeserialize,
{
    if basis.variables == 0 {
        return Err(IpaGeneratorFoldingError::VariableUnderflow);
    }

    basis.validate()?;

    let polynomial_generators =
        fold_ipa_polynomial_generators(&basis.polynomial_generators, challenge)?;
    let evaluation_generators =
        fold_ipa_evaluation_generators(&basis.evaluation_generators, challenge)?;

    Ok(IpaCurveGeneratorBasis::new(
        basis.variables - 1,
        polynomial_generators,
        evaluation_generators,
        basis.blinding_generator.clone(),
    )?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ark_bls12_381::{Fr, G1Projective};
    use ark_ec::{AffineRepr, PrimeGroup};
    use ark_ff::Field;

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

    fn challenge() -> Fr {
        Fr::from(7)
    }

    #[test]
    fn polynomial_generators_fold_with_expected_formula() {
        let generators = vec![point(1), point(2), point(3), point(4)];
        let x = challenge();
        let x_inv = x.inverse().unwrap();

        let folded = fold_ipa_polynomial_generators(&generators, x).unwrap();

        let expected_zero =
            generators[0].affine().into_group() * x_inv + generators[2].affine().into_group() * x;
        let expected_one =
            generators[1].affine().into_group() * x_inv + generators[3].affine().into_group() * x;

        assert_eq!(folded[0].affine().into_group(), expected_zero);
        assert_eq!(folded[1].affine().into_group(), expected_one);
    }

    #[test]
    fn evaluation_generators_fold_with_expected_formula() {
        let generators = vec![point(10), point(11), point(12), point(13)];
        let x = challenge();
        let x_inv = x.inverse().unwrap();

        let folded = fold_ipa_evaluation_generators(&generators, x).unwrap();

        let expected_zero =
            generators[0].affine().into_group() * x + generators[2].affine().into_group() * x_inv;
        let expected_one =
            generators[1].affine().into_group() * x + generators[3].affine().into_group() * x_inv;

        assert_eq!(folded[0].affine().into_group(), expected_zero);
        assert_eq!(folded[1].affine().into_group(), expected_one);
    }

    #[test]
    fn generator_basis_fold_halves_dimension() {
        let basis = basis(3);
        let folded = fold_ipa_generator_basis(&basis, challenge()).unwrap();

        assert_eq!(folded.variables, 2);
        assert_eq!(folded.polynomial_generators.len(), 4);
        assert_eq!(folded.evaluation_generators.len(), 4);
        assert_eq!(folded.blinding_generator, basis.blinding_generator);
        assert!(folded.validate().is_ok());
    }

    #[test]
    fn generator_basis_fold_rejects_zero_variable_basis() {
        let basis = basis(0);

        assert_eq!(
            fold_ipa_generator_basis(&basis, challenge()),
            Err(IpaGeneratorFoldingError::VariableUnderflow)
        );
    }

    #[test]
    fn generator_folding_rejects_zero_challenge() {
        let basis = basis(2);

        assert_eq!(
            fold_ipa_generator_basis(&basis, Fr::from(0)),
            Err(IpaGeneratorFoldingError::ZeroChallenge)
        );
    }

    #[test]
    fn generator_folding_rejects_odd_length() {
        let generators = vec![point(1), point(2), point(3)];

        assert!(matches!(
            fold_ipa_polynomial_generators(&generators, challenge()),
            Err(IpaGeneratorFoldingError::Reduction(
                IpaReductionRoundError::OddInputLength { actual: 3 }
            ))
        ));
    }

    #[test]
    fn generator_folding_rejects_identity_result() {
        let x = challenge();
        let x_inv = x.inverse().unwrap();
        let left = point(1);
        let right_projective = left.affine().into_group() * (-(x_inv / x));
        let right = IpaCurvePoint::from_projective(right_projective).unwrap();

        assert!(matches!(
            fold_ipa_polynomial_generators(&[left, right], x),
            Err(IpaGeneratorFoldingError::Curve(
                IpaCurvePointError::IdentityPoint
            ))
        ));
    }

    #[test]
    fn generator_folding_is_deterministic() {
        let basis = basis(3);

        assert_eq!(
            fold_ipa_generator_basis(&basis, challenge()).unwrap(),
            fold_ipa_generator_basis(&basis, challenge()).unwrap()
        );
    }

    #[test]
    fn generator_folding_changes_with_challenge() {
        let basis = basis(3);

        assert_ne!(
            fold_ipa_generator_basis(&basis, Fr::from(7)).unwrap(),
            fold_ipa_generator_basis(&basis, Fr::from(8)).unwrap()
        );
    }
}
