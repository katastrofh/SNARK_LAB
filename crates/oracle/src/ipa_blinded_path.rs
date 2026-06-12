use ark_ec::{AffineRepr, CurveGroup};
use ark_ff::PrimeField;
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use multilinear::Multilinear;
use snark_lab_transcript::ProofTranscript;

use crate::ipa::IpaCommitment;
use crate::ipa_blinding::{
    extend_ipa_opening_for_blinding, IpaBlindedOpeningExtension, IpaBlindingExtensionError,
};
use crate::ipa_commitment::{commit_ipa_polynomial, IpaCommitmentEquationError};
use crate::ipa_curve::{IpaCurveGeneratorBasis, IpaCurvePoint, IpaCurvePointError};
use crate::ipa_evaluation::{compute_ipa_evaluation_basis, IpaEvaluationBasisError};
use crate::ipa_generators::{expected_ipa_generator_count, IpaGeneratorBasisError};
use crate::ipa_opening_statement::{IpaOpeningStatement, IpaOpeningStatementError};
use crate::ipa_proof::IpaOpeningProof;
use crate::ipa_prover_opening::{prove_ipa_opening, IpaProverOpeningError, IpaProverOpeningOutput};
use crate::ipa_verifier_opening::{verify_ipa_opening, IpaVerifierOpeningError};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IpaBlindedPathError<F: PrimeField> {
    BlindingExtension(IpaBlindingExtensionError),
    Commitment(IpaCommitmentEquationError),
    Curve(IpaCurvePointError),
    EvaluationBasis(IpaEvaluationBasisError),
    GeneratorShape(IpaGeneratorBasisError),
    OpeningStatement(IpaOpeningStatementError<F>),
    Prover(IpaProverOpeningError<F>),
    Verifier(IpaVerifierOpeningError<F>),
    CommitmentWitnessMismatch,
    InvalidExtendedPolynomial,
    ExtendedEvaluationBasisMismatch,
    PointVariableMismatch {
        point_variables: usize,
        basis_variables: usize,
    },
    VariableOverflow,
    PaddingGeneratorCountMismatch {
        label: &'static str,
        expected: usize,
        actual: usize,
    },
}

impl<F: PrimeField> From<IpaBlindingExtensionError> for IpaBlindedPathError<F> {
    fn from(error: IpaBlindingExtensionError) -> Self {
        Self::BlindingExtension(error)
    }
}

impl<F: PrimeField> From<IpaCommitmentEquationError> for IpaBlindedPathError<F> {
    fn from(error: IpaCommitmentEquationError) -> Self {
        Self::Commitment(error)
    }
}

impl<F: PrimeField> From<IpaCurvePointError> for IpaBlindedPathError<F> {
    fn from(error: IpaCurvePointError) -> Self {
        Self::Curve(error)
    }
}

impl<F: PrimeField> From<IpaEvaluationBasisError> for IpaBlindedPathError<F> {
    fn from(error: IpaEvaluationBasisError) -> Self {
        Self::EvaluationBasis(error)
    }
}

impl<F: PrimeField> From<IpaGeneratorBasisError> for IpaBlindedPathError<F> {
    fn from(error: IpaGeneratorBasisError) -> Self {
        Self::GeneratorShape(error)
    }
}

impl<F: PrimeField> From<IpaOpeningStatementError<F>> for IpaBlindedPathError<F> {
    fn from(error: IpaOpeningStatementError<F>) -> Self {
        Self::OpeningStatement(error)
    }
}

impl<F: PrimeField> From<IpaProverOpeningError<F>> for IpaBlindedPathError<F> {
    fn from(error: IpaProverOpeningError<F>) -> Self {
        Self::Prover(error)
    }
}

impl<F: PrimeField> From<IpaVerifierOpeningError<F>> for IpaBlindedPathError<F> {
    fn from(error: IpaVerifierOpeningError<F>) -> Self {
        Self::Verifier(error)
    }
}

pub struct IpaBlindedProverInput<'a, G: CurveGroup> {
    pub basis: &'a IpaCurveGeneratorBasis<G>,
    pub commitment: IpaCommitment,
    pub polynomial: &'a Multilinear<G::ScalarField>,
    pub point: &'a [G::ScalarField],
    pub claimed_value: G::ScalarField,
    pub commitment_blinding: G::ScalarField,
    pub padding_polynomial_generators: Vec<IpaCurvePoint<G>>,
    pub padding_evaluation_generators: Vec<IpaCurvePoint<G>>,
    pub extended_blinding_generator: IpaCurvePoint<G>,
    pub inner_product_generator: &'a IpaCurvePoint<G>,
}

pub struct IpaBlindedVerifierInput<'a, G: CurveGroup> {
    pub basis: &'a IpaCurveGeneratorBasis<G>,
    pub commitment: IpaCommitment,
    pub point: &'a [G::ScalarField],
    pub claimed_value: G::ScalarField,
    pub proof: &'a IpaOpeningProof<G::ScalarField>,
    pub padding_polynomial_generators: Vec<IpaCurvePoint<G>>,
    pub padding_evaluation_generators: Vec<IpaCurvePoint<G>>,
    pub extended_blinding_generator: IpaCurvePoint<G>,
    pub inner_product_generator: &'a IpaCurvePoint<G>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IpaBlindedProverOutput<G: CurveGroup> {
    pub extension: IpaBlindedOpeningExtension<G>,
    pub extended_statement: IpaOpeningStatement<G::ScalarField>,
    pub prover_output: IpaProverOpeningOutput<G>,
}

pub fn blinded_extension_point<F: PrimeField>(point: &[F]) -> Vec<F> {
    let mut extended_point = Vec::with_capacity(point.len() + 1);
    extended_point.extend_from_slice(point);
    extended_point.push(F::zero());
    extended_point
}

fn validate_commitment_matches_witness<G>(
    basis: &IpaCurveGeneratorBasis<G>,
    commitment: &IpaCommitment,
    polynomial: &Multilinear<G::ScalarField>,
    commitment_blinding: G::ScalarField,
) -> Result<(), IpaBlindedPathError<G::ScalarField>>
where
    G: CurveGroup,
    G::Affine: AffineRepr + CanonicalSerialize + CanonicalDeserialize,
{
    let expected_commitment = commit_ipa_polynomial(basis, polynomial, commitment_blinding)?
        .to_opaque_commitment(polynomial.variables())?;

    if expected_commitment.variables != commitment.variables
        || expected_commitment.commitment_bytes != commitment.commitment_bytes
    {
        return Err(IpaBlindedPathError::CommitmentWitnessMismatch);
    }

    Ok(())
}

fn build_extended_generator_basis_for_blinding<G>(
    basis: &IpaCurveGeneratorBasis<G>,
    padding_polynomial_generators: Vec<IpaCurvePoint<G>>,
    padding_evaluation_generators: Vec<IpaCurvePoint<G>>,
    extended_blinding_generator: IpaCurvePoint<G>,
) -> Result<IpaCurveGeneratorBasis<G>, IpaBlindedPathError<G::ScalarField>>
where
    G: CurveGroup,
    G::Affine: AffineRepr + CanonicalSerialize + CanonicalDeserialize,
{
    basis.validate()?;

    let original_len = expected_ipa_generator_count(basis.variables)?;
    let extended_variables = basis
        .variables
        .checked_add(1)
        .ok_or(IpaBlindedPathError::VariableOverflow)?;
    let extended_len = expected_ipa_generator_count(extended_variables)?;

    let expected_polynomial_padding = extended_len
        .checked_sub(original_len + 1)
        .ok_or(IpaBlindedPathError::VariableOverflow)?;
    let expected_evaluation_padding = extended_len
        .checked_sub(original_len)
        .ok_or(IpaBlindedPathError::VariableOverflow)?;

    if padding_polynomial_generators.len() != expected_polynomial_padding {
        return Err(IpaBlindedPathError::PaddingGeneratorCountMismatch {
            label: "polynomial",
            expected: expected_polynomial_padding,
            actual: padding_polynomial_generators.len(),
        });
    }

    if padding_evaluation_generators.len() != expected_evaluation_padding {
        return Err(IpaBlindedPathError::PaddingGeneratorCountMismatch {
            label: "evaluation",
            expected: expected_evaluation_padding,
            actual: padding_evaluation_generators.len(),
        });
    }

    let mut polynomial_generators = basis.polynomial_generators.clone();
    polynomial_generators.push(basis.blinding_generator.clone());
    polynomial_generators.extend(padding_polynomial_generators);

    let mut evaluation_generators = basis.evaluation_generators.clone();
    evaluation_generators.extend(padding_evaluation_generators);

    Ok(IpaCurveGeneratorBasis::new(
        extended_variables,
        polynomial_generators,
        evaluation_generators,
        extended_blinding_generator,
    )?)
}

/// Prove a blinded IPA opening by reducing it to an extended ordinary IPA
/// relation.
///
/// This function validates that the public commitment matches the supplied
/// polynomial and blinding scalar before constructing the proof. It does not
/// fake acceptance for unsupported commitments.
pub fn prove_blinded_ipa_opening<G, T>(
    input: IpaBlindedProverInput<'_, G>,
    transcript: &mut T,
) -> Result<IpaBlindedProverOutput<G>, IpaBlindedPathError<G::ScalarField>>
where
    G: CurveGroup,
    G::Affine: AffineRepr + CanonicalSerialize + CanonicalDeserialize,
    T: ProofTranscript<G::ScalarField>,
{
    if input.point.len() != input.basis.variables {
        return Err(IpaBlindedPathError::PointVariableMismatch {
            point_variables: input.point.len(),
            basis_variables: input.basis.variables,
        });
    }

    validate_commitment_matches_witness(
        input.basis,
        &input.commitment,
        input.polynomial,
        input.commitment_blinding,
    )?;

    let original_evaluation_basis = compute_ipa_evaluation_basis(input.point)?;

    let extension = extend_ipa_opening_for_blinding(
        input.basis,
        input.polynomial,
        &original_evaluation_basis,
        input.commitment_blinding,
        input.padding_polynomial_generators,
        input.padding_evaluation_generators,
        input.extended_blinding_generator,
    )?;

    let extended_point = blinded_extension_point(input.point);
    let extended_evaluation_basis = compute_ipa_evaluation_basis(&extended_point)?;

    if extended_evaluation_basis.basis_evaluations != extension.evaluation_vector {
        return Err(IpaBlindedPathError::ExtendedEvaluationBasisMismatch);
    }

    let extended_polynomial = Multilinear::new(extension.polynomial_vector.clone())
        .map_err(|_| IpaBlindedPathError::InvalidExtendedPolynomial)?;

    let extended_commitment = IpaCommitment {
        variables: extension.extended_variables,
        commitment_bytes: input.commitment.commitment_bytes.clone(),
    };

    let prover_output = prove_ipa_opening(
        &extension.generator_basis,
        extended_commitment,
        &extended_polynomial,
        &extended_point,
        input.claimed_value,
        input.inner_product_generator,
        transcript,
    )?;

    Ok(IpaBlindedProverOutput {
        extension,
        extended_statement: prover_output.statement.clone(),
        prover_output,
    })
}

/// Verify a blinded IPA opening by reconstructing the extended relation and
/// invoking the real IPA verifier.
///
/// The verifier never sees the blinding scalar. It only sees the public
/// commitment, public opening point, claimed value, proof, and the extended
/// generator basis material.
pub fn verify_blinded_ipa_opening<G, T>(
    input: IpaBlindedVerifierInput<'_, G>,
    transcript: &mut T,
) -> Result<(), IpaBlindedPathError<G::ScalarField>>
where
    G: CurveGroup,
    G::Affine: AffineRepr + CanonicalSerialize + CanonicalDeserialize,
    T: ProofTranscript<G::ScalarField>,
{
    if input.point.len() != input.basis.variables {
        return Err(IpaBlindedPathError::PointVariableMismatch {
            point_variables: input.point.len(),
            basis_variables: input.basis.variables,
        });
    }

    let extended_basis = build_extended_generator_basis_for_blinding(
        input.basis,
        input.padding_polynomial_generators,
        input.padding_evaluation_generators,
        input.extended_blinding_generator,
    )?;

    let extended_point = blinded_extension_point(input.point);

    let extended_commitment = IpaCommitment {
        variables: extended_basis.variables,
        commitment_bytes: input.commitment.commitment_bytes.clone(),
    };

    let extended_statement =
        IpaOpeningStatement::new(extended_commitment, extended_point, input.claimed_value)?;

    verify_ipa_opening(
        &extended_basis,
        &extended_statement,
        input.proof,
        input.inner_product_generator,
        transcript,
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ark_bls12_381::{Fr, G1Projective};
    use ark_ec::PrimeGroup;
    use multilinear::Multilinear;
    use snark_lab_transcript::MerlinTranscript;

    use crate::ipa_commitment::commit_ipa_polynomial;
    use crate::ipa_curve::{IpaCurveGeneratorBasis, IpaCurvePoint};
    use crate::ipa_generators::expected_ipa_generator_count;
    use crate::ipa_verifier_opening::IpaVerifierOpeningError;

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

    fn commitment_for(
        basis: &IpaCurveGeneratorBasis<G1Projective>,
        polynomial: &Multilinear<Fr>,
        blinding: Fr,
    ) -> IpaCommitment {
        commit_ipa_polynomial(basis, polynomial, blinding)
            .unwrap()
            .to_opaque_commitment(polynomial.variables())
            .unwrap()
    }

    fn padding_polynomial(variables: usize) -> Vec<IpaCurvePoint<G1Projective>> {
        let original_len = expected_ipa_generator_count(variables).unwrap();
        let extended_len = expected_ipa_generator_count(variables + 1).unwrap();
        let count = extended_len - original_len - 1;

        (0..count).map(|index| point(index as u64 + 2000)).collect()
    }

    fn padding_evaluation(variables: usize) -> Vec<IpaCurvePoint<G1Projective>> {
        let original_len = expected_ipa_generator_count(variables).unwrap();
        let extended_len = expected_ipa_generator_count(variables + 1).unwrap();
        let count = extended_len - original_len;

        (0..count).map(|index| point(index as u64 + 3000)).collect()
    }

    fn prove_blinded_for_values(
        values: &[u64],
        blinding: Fr,
    ) -> (
        IpaCurveGeneratorBasis<G1Projective>,
        IpaCurvePoint<G1Projective>,
        IpaCommitment,
        Vec<Fr>,
        Fr,
        IpaBlindedProverOutput<G1Projective>,
    ) {
        let polynomial = polynomial(values);
        let basis = basis(polynomial.variables());
        let commitment = commitment_for(&basis, &polynomial, blinding);
        let opening_point = vec![Fr::from(3); polynomial.variables()];
        let claimed = polynomial.evaluate(&opening_point).unwrap();
        let inner_product_generator = point(5000);
        let mut transcript = MerlinTranscript::new(b"ipa-blinded-path-test");

        let output = prove_blinded_ipa_opening(
            IpaBlindedProverInput {
                basis: &basis,
                commitment: commitment.clone(),
                polynomial: &polynomial,
                point: &opening_point,
                claimed_value: claimed,
                commitment_blinding: blinding,
                padding_polynomial_generators: padding_polynomial(polynomial.variables()),
                padding_evaluation_generators: padding_evaluation(polynomial.variables()),
                extended_blinding_generator: point(9000),
                inner_product_generator: &inner_product_generator,
            },
            &mut transcript,
        )
        .unwrap();

        (
            basis,
            inner_product_generator,
            commitment,
            opening_point,
            claimed,
            output,
        )
    }

    fn verify_blinded_output(
        basis: &IpaCurveGeneratorBasis<G1Projective>,
        inner_product_generator: &IpaCurvePoint<G1Projective>,
        commitment: IpaCommitment,
        opening_point: &[Fr],
        claimed: Fr,
        output: &IpaBlindedProverOutput<G1Projective>,
    ) -> Result<(), IpaBlindedPathError<Fr>> {
        let mut transcript = MerlinTranscript::new(b"ipa-blinded-path-test");

        verify_blinded_ipa_opening(
            IpaBlindedVerifierInput {
                basis,
                commitment,
                point: opening_point,
                claimed_value: claimed,
                proof: &output.prover_output.proof,
                padding_polynomial_generators: padding_polynomial(basis.variables),
                padding_evaluation_generators: padding_evaluation(basis.variables),
                extended_blinding_generator: point(9000),
                inner_product_generator,
            },
            &mut transcript,
        )
    }

    #[test]
    fn blinded_prover_verifier_accepts_matching_blinded_opening() {
        let (basis, inner_product_generator, commitment, opening_point, claimed, output) =
            prove_blinded_for_values(&[2, 3, 5, 7], Fr::from(9));

        assert_eq!(
            verify_blinded_output(
                &basis,
                &inner_product_generator,
                commitment,
                &opening_point,
                claimed,
                &output,
            ),
            Ok(())
        );
    }

    #[test]
    fn blinded_path_handles_zero_variable_polynomial() {
        let (basis, inner_product_generator, commitment, opening_point, claimed, output) =
            prove_blinded_for_values(&[42], Fr::from(9));

        assert_eq!(output.extension.original_variables, 0);
        assert_eq!(output.extension.extended_variables, 1);
        assert_eq!(
            verify_blinded_output(
                &basis,
                &inner_product_generator,
                commitment,
                &opening_point,
                claimed,
                &output,
            ),
            Ok(())
        );
    }

    #[test]
    fn blinded_prover_rejects_commitment_witness_mismatch() {
        let polynomial = polynomial(&[2, 3, 5, 7]);
        let basis = basis(2);
        let commitment = commitment_for(&basis, &polynomial, Fr::from(10));
        let opening_point = vec![Fr::from(3), Fr::from(3)];
        let claimed = polynomial.evaluate(&opening_point).unwrap();
        let inner_product_generator = point(5000);
        let mut transcript = MerlinTranscript::new(b"ipa-blinded-path-test");

        assert_eq!(
            prove_blinded_ipa_opening(
                IpaBlindedProverInput {
                    basis: &basis,
                    commitment,
                    polynomial: &polynomial,
                    point: &opening_point,
                    claimed_value: claimed,
                    commitment_blinding: Fr::from(9),
                    padding_polynomial_generators: padding_polynomial(2),
                    padding_evaluation_generators: padding_evaluation(2),
                    extended_blinding_generator: point(9000),
                    inner_product_generator: &inner_product_generator,
                },
                &mut transcript,
            ),
            Err(IpaBlindedPathError::CommitmentWitnessMismatch)
        );
    }

    #[test]
    fn blinded_verifier_rejects_wrong_public_commitment() {
        let (basis, inner_product_generator, _commitment, opening_point, claimed, output) =
            prove_blinded_for_values(&[2, 3, 5, 7], Fr::from(9));
        let other_polynomial = polynomial(&[2, 3, 5, 8]);
        let other_commitment = commitment_for(&basis, &other_polynomial, Fr::from(9));

        assert!(matches!(
            verify_blinded_output(
                &basis,
                &inner_product_generator,
                other_commitment,
                &opening_point,
                claimed,
                &output,
            ),
            Err(IpaBlindedPathError::Verifier(
                IpaVerifierOpeningError::FinalCommitmentMismatch
            ))
        ));
    }

    #[test]
    fn blinded_verifier_rejects_tampered_final_scalar() {
        let (basis, inner_product_generator, commitment, opening_point, claimed, mut output) =
            prove_blinded_for_values(&[2, 3, 5, 7], Fr::from(9));

        output.prover_output.proof.final_polynomial_scalar += Fr::from(1);

        assert!(matches!(
            verify_blinded_output(
                &basis,
                &inner_product_generator,
                commitment,
                &opening_point,
                claimed,
                &output,
            ),
            Err(IpaBlindedPathError::Verifier(
                IpaVerifierOpeningError::FinalCommitmentMismatch
            ))
        ));
    }

    #[test]
    fn blinded_verifier_rejects_wrong_padding_count() {
        let (basis, inner_product_generator, commitment, opening_point, claimed, output) =
            prove_blinded_for_values(&[2, 3, 5, 7], Fr::from(9));
        let mut transcript = MerlinTranscript::new(b"ipa-blinded-path-test");

        assert_eq!(
            verify_blinded_ipa_opening(
                IpaBlindedVerifierInput {
                    basis: &basis,
                    commitment,
                    point: &opening_point,
                    claimed_value: claimed,
                    proof: &output.prover_output.proof,
                    padding_polynomial_generators: vec![point(2000)],
                    padding_evaluation_generators: padding_evaluation(2),
                    extended_blinding_generator: point(9000),
                    inner_product_generator: &inner_product_generator,
                },
                &mut transcript,
            ),
            Err(IpaBlindedPathError::PaddingGeneratorCountMismatch {
                label: "polynomial",
                expected: 3,
                actual: 1,
            })
        );
    }

    #[test]
    fn blinded_extended_point_matches_extension_evaluation_vector() {
        let polynomial = polynomial(&[2, 3, 5, 7]);
        let basis = basis(2);
        let opening_point = vec![Fr::from(3), Fr::from(3)];
        let original_evaluation_basis = compute_ipa_evaluation_basis(&opening_point).unwrap();

        let extension = extend_ipa_opening_for_blinding(
            &basis,
            &polynomial,
            &original_evaluation_basis,
            Fr::from(9),
            padding_polynomial(2),
            padding_evaluation(2),
            point(9000),
        )
        .unwrap();

        let extended_point = blinded_extension_point(&opening_point);
        let extended_evaluation_basis = compute_ipa_evaluation_basis(&extended_point).unwrap();

        assert_eq!(
            extension.evaluation_vector,
            extended_evaluation_basis.basis_evaluations
        );
    }
}
