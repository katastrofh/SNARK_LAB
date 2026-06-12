use ark_ff::PrimeField;
use multilinear::{eq_evaluations, Multilinear};
use snark_lab_transcript::ProofTranscript;

use crate::ipa_generators::{expected_ipa_generator_count, IpaGeneratorBasisError};

const IPA_EVALUATION_BASIS_DOMAIN: &[u8] = b"snark-lab/ipa-evaluation-basis/v1";

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IpaEvaluationBasisError {
    GeneratorShape(IpaGeneratorBasisError),
    BasisLengthMismatch {
        expected: usize,
        actual: usize,
    },
    PolynomialVariableMismatch {
        polynomial_variables: usize,
        basis_variables: usize,
    },
}

impl From<IpaGeneratorBasisError> for IpaEvaluationBasisError {
    fn from(error: IpaGeneratorBasisError) -> Self {
        Self::GeneratorShape(error)
    }
}

/// Evaluation-basis vector for an IPA opening claim.
///
/// For a multilinear polynomial table `a` and opening point `z`, the claimed
/// value is:
///
/// ```text
/// f(z) = <a, eq(z, ·)>
/// ```
///
/// The basis vector is ordered to match `Multilinear::evaluations()`.
///
/// Internally, the equality-vector helper enumerates variables in the opposite
/// bit-significance order, so construction reverses the point before expansion.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IpaEvaluationBasis<F: PrimeField> {
    pub variables: usize,
    pub point: Vec<F>,
    pub basis_evaluations: Vec<F>,
}

impl<F: PrimeField> IpaEvaluationBasis<F> {
    pub fn new(point: Vec<F>) -> Result<Self, IpaEvaluationBasisError> {
        let expected = expected_ipa_generator_count(point.len())?;
        let basis_evaluations = evaluation_basis_for_multilinear_order(&point);

        if basis_evaluations.len() != expected {
            return Err(IpaEvaluationBasisError::BasisLengthMismatch {
                expected,
                actual: basis_evaluations.len(),
            });
        }

        Ok(Self {
            variables: point.len(),
            point,
            basis_evaluations,
        })
    }

    pub fn from_parts(
        point: Vec<F>,
        basis_evaluations: Vec<F>,
    ) -> Result<Self, IpaEvaluationBasisError> {
        let expected = expected_ipa_generator_count(point.len())?;

        if basis_evaluations.len() != expected {
            return Err(IpaEvaluationBasisError::BasisLengthMismatch {
                expected,
                actual: basis_evaluations.len(),
            });
        }

        Ok(Self {
            variables: point.len(),
            point,
            basis_evaluations,
        })
    }

    pub fn expected_len(&self) -> Result<usize, IpaEvaluationBasisError> {
        Ok(expected_ipa_generator_count(self.variables)?)
    }

    pub fn inner_product_with_table(&self, table: &[F]) -> Result<F, IpaEvaluationBasisError> {
        let expected = self.expected_len()?;

        if table.len() != expected {
            return Err(IpaEvaluationBasisError::BasisLengthMismatch {
                expected,
                actual: table.len(),
            });
        }

        Ok(table
            .iter()
            .zip(self.basis_evaluations.iter())
            .map(|(left, right)| *left * *right)
            .sum())
    }

    pub fn evaluate_polynomial(
        &self,
        polynomial: &Multilinear<F>,
    ) -> Result<F, IpaEvaluationBasisError> {
        if polynomial.variables() != self.variables {
            return Err(IpaEvaluationBasisError::PolynomialVariableMismatch {
                polynomial_variables: polynomial.variables(),
                basis_variables: self.variables,
            });
        }

        self.inner_product_with_table(polynomial.evaluations())
    }
}

fn evaluation_basis_for_multilinear_order<F: PrimeField>(point: &[F]) -> Vec<F> {
    let mut reversed_point = point.to_vec();
    reversed_point.reverse();
    eq_evaluations(&reversed_point)
}

/// Compute the IPA evaluation-basis vector `eq(z, ·)`.
///
/// The resulting vector is ordered to match `Multilinear::evaluations()`.
pub fn compute_ipa_evaluation_basis<F: PrimeField>(
    point: &[F],
) -> Result<IpaEvaluationBasis<F>, IpaEvaluationBasisError> {
    IpaEvaluationBasis::new(point.to_vec())
}

/// Evaluate a multilinear polynomial using the IPA evaluation-basis inner product.
pub fn evaluate_with_ipa_evaluation_basis<F: PrimeField>(
    polynomial: &Multilinear<F>,
    point: &[F],
) -> Result<F, IpaEvaluationBasisError> {
    compute_ipa_evaluation_basis(point)?.evaluate_polynomial(polynomial)
}

/// Bind the evaluation basis into the Fiat-Shamir transcript.
///
/// This fixes the point and the concrete equality-vector used by the opening
/// proof. It is intentionally separate from commitment binding.
pub fn bind_ipa_evaluation_basis<F: PrimeField, T: ProofTranscript<F>>(
    transcript: &mut T,
    basis: &IpaEvaluationBasis<F>,
) -> Result<(), IpaEvaluationBasisError> {
    let expected = basis.expected_len()?;

    if basis.basis_evaluations.len() != expected {
        return Err(IpaEvaluationBasisError::BasisLengthMismatch {
            expected,
            actual: basis.basis_evaluations.len(),
        });
    }

    transcript.append_domain_separator(IPA_EVALUATION_BASIS_DOMAIN);
    transcript.append_field_modulus();
    transcript.append_u64(b"ipa-evaluation-variables", basis.variables as u64);
    transcript.append_u64(
        b"ipa-evaluation-basis-len",
        basis.basis_evaluations.len() as u64,
    );

    for (index, coordinate) in basis.point.iter().enumerate() {
        transcript.append_u64(b"ipa-evaluation-point-index", index as u64);
        transcript.append_field_element(b"ipa-evaluation-point-coordinate", coordinate);
    }

    for (index, value) in basis.basis_evaluations.iter().enumerate() {
        transcript.append_u64(b"ipa-evaluation-basis-index", index as u64);
        transcript.append_field_element(b"ipa-evaluation-basis-value", value);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ark_bls12_381::Fr;
    use ark_ff::Field;
    use snark_lab_transcript::{MerlinTranscript, ProofTranscript};

    fn polynomial(values: &[u64]) -> Multilinear<Fr> {
        Multilinear::new(values.iter().copied().map(Fr::from).collect()).unwrap()
    }

    fn challenge_for_basis(basis: &IpaEvaluationBasis<Fr>) -> Fr {
        let mut transcript = MerlinTranscript::new(b"ipa-evaluation-basis-test");

        bind_ipa_evaluation_basis::<Fr, _>(&mut transcript, basis).unwrap();

        transcript.challenge_scalar(b"after-evaluation-basis")
    }

    #[test]
    fn evaluation_basis_has_power_of_two_length() {
        let basis =
            compute_ipa_evaluation_basis::<Fr>(&[Fr::from(3), Fr::from(5), Fr::from(7)]).unwrap();

        assert_eq!(basis.variables, 3);
        assert_eq!(basis.basis_evaluations.len(), 8);
        assert_eq!(basis.basis_evaluations.iter().copied().sum::<Fr>(), Fr::ONE);
    }

    #[test]
    fn evaluation_basis_matches_multilinear_evaluate() {
        let polynomial = polynomial(&[2, 3, 5, 7, 11, 13, 17, 19]);
        let point = [Fr::from(3), Fr::from(5), Fr::from(7)];

        let expected = polynomial.evaluate(&point).unwrap();
        let via_basis = evaluate_with_ipa_evaluation_basis(&polynomial, &point).unwrap();

        assert_eq!(via_basis, expected);
    }

    #[test]
    fn evaluation_basis_rejects_manual_wrong_length() {
        assert_eq!(
            IpaEvaluationBasis::<Fr>::from_parts(vec![Fr::from(3), Fr::from(5)], vec![Fr::ONE]),
            Err(IpaEvaluationBasisError::BasisLengthMismatch {
                expected: 4,
                actual: 1
            })
        );
    }

    #[test]
    fn evaluation_basis_rejects_polynomial_variable_mismatch() {
        let polynomial = polynomial(&[2, 3, 5, 7]);
        let basis = compute_ipa_evaluation_basis::<Fr>(&[Fr::from(3)]).unwrap();

        assert_eq!(
            basis.evaluate_polynomial(&polynomial),
            Err(IpaEvaluationBasisError::PolynomialVariableMismatch {
                polynomial_variables: 2,
                basis_variables: 1
            })
        );
    }

    #[test]
    fn evaluation_basis_inner_product_rejects_wrong_table_length() {
        let basis = compute_ipa_evaluation_basis::<Fr>(&[Fr::from(3), Fr::from(5)]).unwrap();

        assert_eq!(
            basis.inner_product_with_table(&[Fr::ONE]),
            Err(IpaEvaluationBasisError::BasisLengthMismatch {
                expected: 4,
                actual: 1
            })
        );
    }

    #[test]
    fn evaluation_basis_binding_is_deterministic() {
        let basis = compute_ipa_evaluation_basis::<Fr>(&[Fr::from(3), Fr::from(5)]).unwrap();

        assert_eq!(challenge_for_basis(&basis), challenge_for_basis(&basis));
    }

    #[test]
    fn evaluation_basis_binding_changes_when_point_changes() {
        let a = compute_ipa_evaluation_basis::<Fr>(&[Fr::from(3), Fr::from(5)]).unwrap();
        let b = compute_ipa_evaluation_basis::<Fr>(&[Fr::from(3), Fr::from(6)]).unwrap();

        assert_ne!(challenge_for_basis(&a), challenge_for_basis(&b));
    }

    #[test]
    fn zero_variable_evaluation_basis_is_single_one() {
        let basis = compute_ipa_evaluation_basis::<Fr>(&[]).unwrap();

        assert_eq!(basis.variables, 0);
        assert_eq!(basis.basis_evaluations, vec![Fr::ONE]);
    }
}
