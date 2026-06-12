use ark_ec::{AffineRepr, CurveGroup};
use ark_ff::Zero;
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use multilinear::Multilinear;

use crate::ipa_curve::{IpaCurveGeneratorBasis, IpaCurvePoint, IpaCurvePointError};
use crate::ipa_evaluation::IpaEvaluationBasis;
use crate::ipa_generators::{expected_ipa_generator_count, IpaGeneratorBasisError};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IpaBlindingExtensionError {
    GeneratorShape(IpaGeneratorBasisError),
    Curve(IpaCurvePointError),
    VariableOverflow,
    BasisVariableMismatch {
        basis_variables: usize,
        polynomial_variables: usize,
    },
    EvaluationBasisVariableMismatch {
        basis_variables: usize,
        polynomial_variables: usize,
    },
    PolynomialLengthMismatch {
        expected: usize,
        actual: usize,
    },
    EvaluationBasisLengthMismatch {
        expected: usize,
        actual: usize,
    },
    PaddingGeneratorCountMismatch {
        label: &'static str,
        expected: usize,
        actual: usize,
    },
}

impl From<IpaGeneratorBasisError> for IpaBlindingExtensionError {
    fn from(error: IpaGeneratorBasisError) -> Self {
        Self::GeneratorShape(error)
    }
}

impl From<IpaCurvePointError> for IpaBlindingExtensionError {
    fn from(error: IpaCurvePointError) -> Self {
        Self::Curve(error)
    }
}

/// Extended IPA witness for a hiding/blinded commitment opening.
///
/// A blinded commitment has the form:
///
/// ```text
/// C = <a, G> + rB
/// ```
///
/// This module converts it into an ordinary IPA relation by extending the
/// witness vector and generator vector:
///
/// ```text
/// a_ext = (a, r, 0, ..., 0)
/// G_ext = (G, B, padding generators)
/// b_ext = (eq(z, ·), 0, ..., 0)
/// ```
///
/// The opening value remains:
///
/// ```text
/// v = <a_ext, b_ext> = <a, eq(z, ·)>
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IpaBlindedOpeningExtension<G: CurveGroup> {
    pub original_variables: usize,
    pub extended_variables: usize,
    pub commitment_blinding: G::ScalarField,
    pub polynomial_vector: Vec<G::ScalarField>,
    pub evaluation_vector: Vec<G::ScalarField>,
    pub generator_basis: IpaCurveGeneratorBasis<G>,
}

fn validate_original_shapes<G>(
    basis: &IpaCurveGeneratorBasis<G>,
    polynomial: &Multilinear<G::ScalarField>,
    evaluation_basis: &IpaEvaluationBasis<G::ScalarField>,
) -> Result<usize, IpaBlindingExtensionError>
where
    G: CurveGroup,
    G::Affine: AffineRepr + CanonicalSerialize + CanonicalDeserialize,
{
    basis.validate()?;

    let polynomial_variables = polynomial.variables();

    if basis.variables != polynomial_variables {
        return Err(IpaBlindingExtensionError::BasisVariableMismatch {
            basis_variables: basis.variables,
            polynomial_variables,
        });
    }

    if evaluation_basis.variables != polynomial_variables {
        return Err(IpaBlindingExtensionError::EvaluationBasisVariableMismatch {
            basis_variables: evaluation_basis.variables,
            polynomial_variables,
        });
    }

    let expected_original_len = expected_ipa_generator_count(polynomial_variables)?;

    if polynomial.evaluations().len() != expected_original_len {
        return Err(IpaBlindingExtensionError::PolynomialLengthMismatch {
            expected: expected_original_len,
            actual: polynomial.evaluations().len(),
        });
    }

    if evaluation_basis.basis_evaluations.len() != expected_original_len {
        return Err(IpaBlindingExtensionError::EvaluationBasisLengthMismatch {
            expected: expected_original_len,
            actual: evaluation_basis.basis_evaluations.len(),
        });
    }

    Ok(expected_original_len)
}

/// Extend a blinded opening into a power-of-two IPA relation.
///
/// `padding_polynomial_generators` and `padding_evaluation_generators` must be
/// generated independently with proper domain separation by the caller. The
/// `extended_blinding_generator` is only the carried-forward hiding generator
/// for the extended basis object and must be distinct from every other generator.
pub fn extend_ipa_opening_for_blinding<G>(
    basis: &IpaCurveGeneratorBasis<G>,
    polynomial: &Multilinear<G::ScalarField>,
    evaluation_basis: &IpaEvaluationBasis<G::ScalarField>,
    commitment_blinding: G::ScalarField,
    padding_polynomial_generators: Vec<IpaCurvePoint<G>>,
    padding_evaluation_generators: Vec<IpaCurvePoint<G>>,
    extended_blinding_generator: IpaCurvePoint<G>,
) -> Result<IpaBlindedOpeningExtension<G>, IpaBlindingExtensionError>
where
    G: CurveGroup,
    G::Affine: AffineRepr + CanonicalSerialize + CanonicalDeserialize,
{
    let original_len = validate_original_shapes(basis, polynomial, evaluation_basis)?;
    let original_variables = polynomial.variables();
    let extended_variables = original_variables
        .checked_add(1)
        .ok_or(IpaBlindingExtensionError::VariableOverflow)?;
    let extended_len = expected_ipa_generator_count(extended_variables)?;

    let expected_polynomial_padding = extended_len
        .checked_sub(original_len + 1)
        .ok_or(IpaBlindingExtensionError::VariableOverflow)?;
    let expected_evaluation_padding = extended_len
        .checked_sub(original_len)
        .ok_or(IpaBlindingExtensionError::VariableOverflow)?;

    if padding_polynomial_generators.len() != expected_polynomial_padding {
        return Err(IpaBlindingExtensionError::PaddingGeneratorCountMismatch {
            label: "polynomial",
            expected: expected_polynomial_padding,
            actual: padding_polynomial_generators.len(),
        });
    }

    if padding_evaluation_generators.len() != expected_evaluation_padding {
        return Err(IpaBlindingExtensionError::PaddingGeneratorCountMismatch {
            label: "evaluation",
            expected: expected_evaluation_padding,
            actual: padding_evaluation_generators.len(),
        });
    }

    let mut polynomial_vector = polynomial.evaluations().to_vec();
    polynomial_vector.push(commitment_blinding);
    polynomial_vector.extend(std::iter::repeat_n(
        G::ScalarField::zero(),
        expected_polynomial_padding,
    ));

    let mut evaluation_vector = evaluation_basis.basis_evaluations.clone();
    evaluation_vector.extend(std::iter::repeat_n(
        G::ScalarField::zero(),
        expected_evaluation_padding,
    ));

    let mut polynomial_generators = basis.polynomial_generators.clone();
    polynomial_generators.push(basis.blinding_generator.clone());
    polynomial_generators.extend(padding_polynomial_generators);

    let mut evaluation_generators = basis.evaluation_generators.clone();
    evaluation_generators.extend(padding_evaluation_generators);

    let generator_basis = IpaCurveGeneratorBasis::new(
        extended_variables,
        polynomial_generators,
        evaluation_generators,
        extended_blinding_generator,
    )?;

    Ok(IpaBlindedOpeningExtension {
        original_variables,
        extended_variables,
        commitment_blinding,
        polynomial_vector,
        evaluation_vector,
        generator_basis,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use ark_bls12_381::{Fr, G1Projective};
    use ark_ec::PrimeGroup;

    use crate::ipa_evaluation::compute_ipa_evaluation_basis;
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

    fn polynomial(values: &[u64]) -> Multilinear<Fr> {
        Multilinear::new(values.iter().copied().map(Fr::from).collect()).unwrap()
    }

    fn inner_product(left: &[Fr], right: &[Fr]) -> Fr {
        left.iter()
            .zip(right.iter())
            .map(|(left_value, right_value)| *left_value * *right_value)
            .sum()
    }

    #[test]
    fn blinded_extension_preserves_opening_value() {
        let polynomial = polynomial(&[2, 3, 5, 7]);
        let basis = basis(2);
        let point_coordinates = vec![Fr::from(3), Fr::from(5)];
        let evaluation_basis = compute_ipa_evaluation_basis(&point_coordinates).unwrap();
        let claimed = polynomial.evaluate(&point_coordinates).unwrap();

        let extension = extend_ipa_opening_for_blinding(
            &basis,
            &polynomial,
            &evaluation_basis,
            Fr::from(123),
            vec![point(2000), point(2001), point(2002)],
            vec![point(3000), point(3001), point(3002), point(3003)],
            point(9000),
        )
        .unwrap();

        assert_eq!(extension.original_variables, 2);
        assert_eq!(extension.extended_variables, 3);
        assert_eq!(extension.polynomial_vector.len(), 8);
        assert_eq!(extension.evaluation_vector.len(), 8);
        assert_eq!(extension.polynomial_vector[4], Fr::from(123));
        assert_eq!(extension.evaluation_vector[4], Fr::from(0));
        assert_eq!(
            inner_product(&extension.polynomial_vector, &extension.evaluation_vector),
            claimed
        );
        assert!(extension.generator_basis.validate().is_ok());
    }

    #[test]
    fn blinded_extension_places_original_blinding_generator_as_polynomial_generator() {
        let polynomial = polynomial(&[2, 3, 5, 7]);
        let basis = basis(2);
        let evaluation_basis = compute_ipa_evaluation_basis(&[Fr::from(3), Fr::from(5)]).unwrap();

        let extension = extend_ipa_opening_for_blinding(
            &basis,
            &polynomial,
            &evaluation_basis,
            Fr::from(123),
            vec![point(2000), point(2001), point(2002)],
            vec![point(3000), point(3001), point(3002), point(3003)],
            point(9000),
        )
        .unwrap();

        assert_eq!(
            extension.generator_basis.polynomial_generators[4],
            basis.blinding_generator
        );
    }

    #[test]
    fn blinded_extension_rejects_basis_variable_mismatch() {
        let polynomial = polynomial(&[2, 3, 5, 7]);
        let bad_basis = basis(1);
        let evaluation_basis = compute_ipa_evaluation_basis(&[Fr::from(3), Fr::from(5)]).unwrap();

        assert_eq!(
            extend_ipa_opening_for_blinding(
                &bad_basis,
                &polynomial,
                &evaluation_basis,
                Fr::from(123),
                Vec::new(),
                Vec::new(),
                point(9000),
            ),
            Err(IpaBlindingExtensionError::BasisVariableMismatch {
                basis_variables: 1,
                polynomial_variables: 2,
            })
        );
    }

    #[test]
    fn blinded_extension_rejects_evaluation_basis_variable_mismatch() {
        let polynomial = polynomial(&[2, 3, 5, 7]);
        let basis = basis(2);
        let bad_evaluation_basis = compute_ipa_evaluation_basis(&[Fr::from(3)]).unwrap();

        assert_eq!(
            extend_ipa_opening_for_blinding(
                &basis,
                &polynomial,
                &bad_evaluation_basis,
                Fr::from(123),
                vec![point(2000), point(2001), point(2002)],
                vec![point(3000), point(3001), point(3002), point(3003)],
                point(9000),
            ),
            Err(IpaBlindingExtensionError::EvaluationBasisVariableMismatch {
                basis_variables: 1,
                polynomial_variables: 2,
            })
        );
    }

    #[test]
    fn blinded_extension_rejects_wrong_padding_count() {
        let polynomial = polynomial(&[2, 3, 5, 7]);
        let basis = basis(2);
        let evaluation_basis = compute_ipa_evaluation_basis(&[Fr::from(3), Fr::from(5)]).unwrap();

        assert_eq!(
            extend_ipa_opening_for_blinding(
                &basis,
                &polynomial,
                &evaluation_basis,
                Fr::from(123),
                vec![point(2000)],
                vec![point(3000), point(3001), point(3002), point(3003)],
                point(9000),
            ),
            Err(IpaBlindingExtensionError::PaddingGeneratorCountMismatch {
                label: "polynomial",
                expected: 3,
                actual: 1,
            })
        );
    }

    #[test]
    fn blinded_extension_rejects_duplicate_extended_blinding_generator() {
        let polynomial = polynomial(&[2, 3, 5, 7]);
        let basis = basis(2);
        let evaluation_basis = compute_ipa_evaluation_basis(&[Fr::from(3), Fr::from(5)]).unwrap();

        assert!(matches!(
            extend_ipa_opening_for_blinding(
                &basis,
                &polynomial,
                &evaluation_basis,
                Fr::from(123),
                vec![point(2000), point(2001), point(2002)],
                vec![point(3000), point(3001), point(3002), point(3003)],
                point(1),
            ),
            Err(IpaBlindingExtensionError::Curve(
                IpaCurvePointError::DuplicatePoint { .. }
            ))
        ));
    }

    #[test]
    fn zero_variable_blinded_extension_has_two_entries() {
        let polynomial = polynomial(&[42]);
        let basis = basis(0);
        let evaluation_basis = compute_ipa_evaluation_basis::<Fr>(&[]).unwrap();

        let extension = extend_ipa_opening_for_blinding(
            &basis,
            &polynomial,
            &evaluation_basis,
            Fr::from(123),
            Vec::new(),
            vec![point(3000)],
            point(9000),
        )
        .unwrap();

        assert_eq!(extension.original_variables, 0);
        assert_eq!(extension.extended_variables, 1);
        assert_eq!(
            extension.polynomial_vector,
            vec![Fr::from(42), Fr::from(123)]
        );
        assert_eq!(extension.evaluation_vector, vec![Fr::from(1), Fr::from(0)]);
    }
}
