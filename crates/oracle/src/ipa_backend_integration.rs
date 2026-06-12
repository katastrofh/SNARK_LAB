use ark_ec::{AffineRepr, CurveGroup};
use ark_ff::PrimeField;
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use multilinear::Multilinear;
use snark_lab_transcript::ProofTranscript;

use crate::ipa::IpaCommitment;
use crate::ipa_blinded_path::{
    prove_blinded_ipa_opening, verify_blinded_ipa_opening, IpaBlindedPathError,
    IpaBlindedProverInput, IpaBlindedVerifierInput,
};
use crate::ipa_commitment::{commit_ipa_polynomial, IpaCommitmentEquationError};
use crate::ipa_curve::{IpaCurveGeneratorBasis, IpaCurvePoint, IpaCurvePointError};
use crate::ipa_generators::{expected_ipa_generator_count, IpaGeneratorBasisError};
use crate::ipa_proof::IpaOpeningProof;
use crate::pcs::{validate_opening_point, validate_supported_variables, PcsShapeError};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IpaBackendIntegrationError<F: PrimeField> {
    Shape(PcsShapeError),
    Commitment(IpaCommitmentEquationError),
    Curve(IpaCurvePointError),
    GeneratorShape(IpaGeneratorBasisError),
    BlindedPath(IpaBlindedPathError<F>),
    EvaluationFailed,
    KeyVariableMismatch {
        key_variables: usize,
        polynomial_variables: usize,
    },
    CommitmentVariableMismatch {
        key_variables: usize,
        commitment_variables: usize,
    },
    PaddingGeneratorCountMismatch {
        label: &'static str,
        expected: usize,
        actual: usize,
    },
    VariableOverflow,
}

impl<F: PrimeField> From<PcsShapeError> for IpaBackendIntegrationError<F> {
    fn from(error: PcsShapeError) -> Self {
        Self::Shape(error)
    }
}

impl<F: PrimeField> From<IpaCommitmentEquationError> for IpaBackendIntegrationError<F> {
    fn from(error: IpaCommitmentEquationError) -> Self {
        Self::Commitment(error)
    }
}

impl<F: PrimeField> From<IpaCurvePointError> for IpaBackendIntegrationError<F> {
    fn from(error: IpaCurvePointError) -> Self {
        Self::Curve(error)
    }
}

impl<F: PrimeField> From<IpaGeneratorBasisError> for IpaBackendIntegrationError<F> {
    fn from(error: IpaGeneratorBasisError) -> Self {
        Self::GeneratorShape(error)
    }
}

impl<F: PrimeField> From<IpaBlindedPathError<F>> for IpaBackendIntegrationError<F> {
    fn from(error: IpaBlindedPathError<F>) -> Self {
        Self::BlindedPath(error)
    }
}

/// Typed prover key for the real IPA backend path.
///
/// This is exact-size key material. It does not pretend to support arbitrary
/// smaller dimensions without separately derived generator material.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IpaIntegratedProverKey<G: CurveGroup> {
    pub supported_variables: usize,
    pub basis: IpaCurveGeneratorBasis<G>,
    pub inner_product_generator: IpaCurvePoint<G>,
    pub padding_polynomial_generators: Vec<IpaCurvePoint<G>>,
    pub padding_evaluation_generators: Vec<IpaCurvePoint<G>>,
    pub extended_blinding_generator: IpaCurvePoint<G>,
}

/// Typed verifier key for the real IPA backend path.
///
/// Contains only public generator material.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IpaIntegratedVerifierKey<G: CurveGroup> {
    pub supported_variables: usize,
    pub basis: IpaCurveGeneratorBasis<G>,
    pub inner_product_generator: IpaCurvePoint<G>,
    pub padding_polynomial_generators: Vec<IpaCurvePoint<G>>,
    pub padding_evaluation_generators: Vec<IpaCurvePoint<G>>,
    pub extended_blinding_generator: IpaCurvePoint<G>,
}

pub type IpaIntegratedKeyPair<G> = (IpaIntegratedProverKey<G>, IpaIntegratedVerifierKey<G>);

/// Commitment plus prover-side witness material.
///
/// The blinding scalar is secret prover witness material. Do not serialize it as
/// part of a public proof.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IpaIntegratedCommitmentWitness<F: PrimeField> {
    pub commitment: IpaCommitment,
    pub blinding: F,
}

/// Typed opening returned by the integrated backend.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IpaIntegratedOpening<F: PrimeField> {
    pub claimed_value: F,
    pub proof: IpaOpeningProof<F>,
}

fn validate_exact_variables<F: PrimeField>(
    supported_variables: usize,
    polynomial_variables: usize,
) -> Result<(), IpaBackendIntegrationError<F>> {
    validate_supported_variables(supported_variables, polynomial_variables)?;

    if supported_variables != polynomial_variables {
        return Err(IpaBackendIntegrationError::KeyVariableMismatch {
            key_variables: supported_variables,
            polynomial_variables,
        });
    }

    Ok(())
}

fn validate_key_material<G>(
    basis: &IpaCurveGeneratorBasis<G>,
    padding_polynomial_generators: &[IpaCurvePoint<G>],
    padding_evaluation_generators: &[IpaCurvePoint<G>],
) -> Result<(), IpaBackendIntegrationError<G::ScalarField>>
where
    G: CurveGroup,
    G::Affine: AffineRepr + CanonicalSerialize + CanonicalDeserialize,
{
    basis.validate()?;

    let original_len = expected_ipa_generator_count(basis.variables)?;
    let extended_variables = basis
        .variables
        .checked_add(1)
        .ok_or(IpaBackendIntegrationError::VariableOverflow)?;
    let extended_len = expected_ipa_generator_count(extended_variables)?;

    let expected_polynomial_padding = extended_len
        .checked_sub(original_len + 1)
        .ok_or(IpaBackendIntegrationError::VariableOverflow)?;
    let expected_evaluation_padding = extended_len
        .checked_sub(original_len)
        .ok_or(IpaBackendIntegrationError::VariableOverflow)?;

    if padding_polynomial_generators.len() != expected_polynomial_padding {
        return Err(IpaBackendIntegrationError::PaddingGeneratorCountMismatch {
            label: "polynomial",
            expected: expected_polynomial_padding,
            actual: padding_polynomial_generators.len(),
        });
    }

    if padding_evaluation_generators.len() != expected_evaluation_padding {
        return Err(IpaBackendIntegrationError::PaddingGeneratorCountMismatch {
            label: "evaluation",
            expected: expected_evaluation_padding,
            actual: padding_evaluation_generators.len(),
        });
    }

    Ok(())
}

/// Build exact-size integrated prover/verifier keys from supplied generator material.
///
/// The caller is responsible for deriving or loading the generators with proper
/// domain separation. This function validates shape, identity, and duplicate
/// errors through the typed generator basis.
pub fn trim_ipa_integrated_keys<G>(
    basis: IpaCurveGeneratorBasis<G>,
    inner_product_generator: IpaCurvePoint<G>,
    padding_polynomial_generators: Vec<IpaCurvePoint<G>>,
    padding_evaluation_generators: Vec<IpaCurvePoint<G>>,
    extended_blinding_generator: IpaCurvePoint<G>,
) -> Result<IpaIntegratedKeyPair<G>, IpaBackendIntegrationError<G::ScalarField>>
where
    G: CurveGroup,
    G::Affine: AffineRepr + CanonicalSerialize + CanonicalDeserialize,
{
    validate_key_material(
        &basis,
        &padding_polynomial_generators,
        &padding_evaluation_generators,
    )?;

    let prover_key = IpaIntegratedProverKey {
        supported_variables: basis.variables,
        basis: basis.clone(),
        inner_product_generator: inner_product_generator.clone(),
        padding_polynomial_generators: padding_polynomial_generators.clone(),
        padding_evaluation_generators: padding_evaluation_generators.clone(),
        extended_blinding_generator: extended_blinding_generator.clone(),
    };

    let verifier_key = IpaIntegratedVerifierKey {
        supported_variables: basis.variables,
        basis,
        inner_product_generator,
        padding_polynomial_generators,
        padding_evaluation_generators,
        extended_blinding_generator,
    };

    Ok((prover_key, verifier_key))
}

/// Commit with an explicit hiding blinding scalar.
///
/// This is not deterministic-only and does not manufacture blinding internally.
pub fn commit_ipa_backend<G>(
    prover_key: &IpaIntegratedProverKey<G>,
    polynomial: &Multilinear<G::ScalarField>,
    blinding: G::ScalarField,
) -> Result<
    IpaIntegratedCommitmentWitness<G::ScalarField>,
    IpaBackendIntegrationError<G::ScalarField>,
>
where
    G: CurveGroup,
    G::Affine: AffineRepr + CanonicalSerialize + CanonicalDeserialize,
{
    validate_exact_variables(prover_key.supported_variables, polynomial.variables())?;

    let commitment = commit_ipa_polynomial(&prover_key.basis, polynomial, blinding)?
        .to_opaque_commitment(polynomial.variables())?;

    Ok(IpaIntegratedCommitmentWitness {
        commitment,
        blinding,
    })
}

/// Open a commitment using the real blinded IPA path.
pub fn open_ipa_backend<G, T>(
    prover_key: &IpaIntegratedProverKey<G>,
    witness: &IpaIntegratedCommitmentWitness<G::ScalarField>,
    polynomial: &Multilinear<G::ScalarField>,
    point: &[G::ScalarField],
    transcript: &mut T,
) -> Result<IpaIntegratedOpening<G::ScalarField>, IpaBackendIntegrationError<G::ScalarField>>
where
    G: CurveGroup,
    G::Affine: AffineRepr + CanonicalSerialize + CanonicalDeserialize,
    T: ProofTranscript<G::ScalarField>,
{
    validate_exact_variables(prover_key.supported_variables, polynomial.variables())?;
    validate_opening_point(polynomial.variables(), point.len())?;

    let claimed_value = polynomial
        .evaluate(point)
        .map_err(|_| IpaBackendIntegrationError::EvaluationFailed)?;

    let output = prove_blinded_ipa_opening(
        IpaBlindedProverInput {
            basis: &prover_key.basis,
            commitment: witness.commitment.clone(),
            polynomial,
            point,
            claimed_value,
            commitment_blinding: witness.blinding,
            padding_polynomial_generators: prover_key.padding_polynomial_generators.clone(),
            padding_evaluation_generators: prover_key.padding_evaluation_generators.clone(),
            extended_blinding_generator: prover_key.extended_blinding_generator.clone(),
            inner_product_generator: &prover_key.inner_product_generator,
        },
        transcript,
    )?;

    Ok(IpaIntegratedOpening {
        claimed_value,
        proof: output.prover_output.proof,
    })
}

/// Verify an opening using the real blinded IPA verifier path.
pub fn verify_ipa_backend<G, T>(
    verifier_key: &IpaIntegratedVerifierKey<G>,
    commitment: &IpaCommitment,
    point: &[G::ScalarField],
    opening: &IpaIntegratedOpening<G::ScalarField>,
    transcript: &mut T,
) -> Result<G::ScalarField, IpaBackendIntegrationError<G::ScalarField>>
where
    G: CurveGroup,
    G::Affine: AffineRepr + CanonicalSerialize + CanonicalDeserialize,
    T: ProofTranscript<G::ScalarField>,
{
    validate_supported_variables(verifier_key.supported_variables, commitment.variables)?;
    validate_opening_point(commitment.variables, point.len())?;

    if commitment.variables != verifier_key.supported_variables {
        return Err(IpaBackendIntegrationError::CommitmentVariableMismatch {
            key_variables: verifier_key.supported_variables,
            commitment_variables: commitment.variables,
        });
    }

    verify_blinded_ipa_opening(
        IpaBlindedVerifierInput {
            basis: &verifier_key.basis,
            commitment: commitment.clone(),
            point,
            claimed_value: opening.claimed_value,
            proof: &opening.proof,
            padding_polynomial_generators: verifier_key.padding_polynomial_generators.clone(),
            padding_evaluation_generators: verifier_key.padding_evaluation_generators.clone(),
            extended_blinding_generator: verifier_key.extended_blinding_generator.clone(),
            inner_product_generator: &verifier_key.inner_product_generator,
        },
        transcript,
    )?;

    Ok(opening.claimed_value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ark_bls12_381::{Fr, G1Projective};
    use ark_ec::PrimeGroup;
    use multilinear::Multilinear;
    use snark_lab_transcript::MerlinTranscript;

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

    fn padding_polynomial(variables: usize) -> Vec<IpaCurvePoint<G1Projective>> {
        let original_len = expected_ipa_generator_count(variables).unwrap();
        let extended_len = expected_ipa_generator_count(variables + 1).unwrap();

        (0..(extended_len - original_len - 1))
            .map(|index| point(index as u64 + 2000))
            .collect()
    }

    fn padding_evaluation(variables: usize) -> Vec<IpaCurvePoint<G1Projective>> {
        let original_len = expected_ipa_generator_count(variables).unwrap();
        let extended_len = expected_ipa_generator_count(variables + 1).unwrap();

        (0..(extended_len - original_len))
            .map(|index| point(index as u64 + 3000))
            .collect()
    }

    fn keys(
        variables: usize,
    ) -> (
        IpaIntegratedProverKey<G1Projective>,
        IpaIntegratedVerifierKey<G1Projective>,
    ) {
        trim_ipa_integrated_keys(
            basis(variables),
            point(5000),
            padding_polynomial(variables),
            padding_evaluation(variables),
            point(9000),
        )
        .unwrap()
    }

    fn polynomial(values: &[u64]) -> Multilinear<Fr> {
        Multilinear::new(values.iter().copied().map(Fr::from).collect()).unwrap()
    }

    #[test]
    fn integrated_backend_commits_opens_and_verifies_blinded_polynomial() {
        let polynomial = polynomial(&[2, 3, 5, 7]);
        let point_coordinates = vec![Fr::from(3), Fr::from(3)];
        let expected_value = polynomial.evaluate(&point_coordinates).unwrap();
        let (prover_key, verifier_key) = keys(2);

        let witness = commit_ipa_backend(&prover_key, &polynomial, Fr::from(9)).unwrap();

        let mut prover_transcript = MerlinTranscript::new(b"ipa-integrated-backend-test");
        let opening = open_ipa_backend(
            &prover_key,
            &witness,
            &polynomial,
            &point_coordinates,
            &mut prover_transcript,
        )
        .unwrap();

        let mut verifier_transcript = MerlinTranscript::new(b"ipa-integrated-backend-test");
        let verified_value = verify_ipa_backend(
            &verifier_key,
            &witness.commitment,
            &point_coordinates,
            &opening,
            &mut verifier_transcript,
        )
        .unwrap();

        assert_eq!(opening.claimed_value, expected_value);
        assert_eq!(verified_value, expected_value);
    }

    #[test]
    fn integrated_backend_uses_explicit_nonzero_blinding() {
        let polynomial = polynomial(&[2, 3, 5, 7]);
        let point_coordinates = vec![Fr::from(3), Fr::from(3)];
        let (prover_key, verifier_key) = keys(2);

        let first = commit_ipa_backend(&prover_key, &polynomial, Fr::from(9)).unwrap();
        let second = commit_ipa_backend(&prover_key, &polynomial, Fr::from(10)).unwrap();

        assert_ne!(first.commitment, second.commitment);

        for witness in [first, second] {
            let mut prover_transcript = MerlinTranscript::new(b"ipa-integrated-backend-test");
            let opening = open_ipa_backend(
                &prover_key,
                &witness,
                &polynomial,
                &point_coordinates,
                &mut prover_transcript,
            )
            .unwrap();

            let mut verifier_transcript = MerlinTranscript::new(b"ipa-integrated-backend-test");
            assert!(verify_ipa_backend(
                &verifier_key,
                &witness.commitment,
                &point_coordinates,
                &opening,
                &mut verifier_transcript,
            )
            .is_ok());
        }
    }

    #[test]
    fn integrated_backend_rejects_wrong_point_length() {
        let polynomial = polynomial(&[2, 3, 5, 7]);
        let (prover_key, _verifier_key) = keys(2);
        let witness = commit_ipa_backend(&prover_key, &polynomial, Fr::from(9)).unwrap();
        let mut transcript = MerlinTranscript::new(b"ipa-integrated-backend-test");

        assert_eq!(
            open_ipa_backend(
                &prover_key,
                &witness,
                &polynomial,
                &[Fr::from(1)],
                &mut transcript
            ),
            Err(IpaBackendIntegrationError::Shape(
                PcsShapeError::PointLengthMismatch {
                    expected: 2,
                    actual: 1
                }
            ))
        );
    }

    #[test]
    fn integrated_backend_rejects_wrong_polynomial_size() {
        let polynomial = polynomial(&[2, 3, 5, 7, 11, 13, 17, 19]);
        let (prover_key, _verifier_key) = keys(2);

        assert_eq!(
            commit_ipa_backend(&prover_key, &polynomial, Fr::from(9)),
            Err(IpaBackendIntegrationError::Shape(
                PcsShapeError::UnsupportedVariableCount {
                    requested: 3,
                    supported: 2
                }
            ))
        );
    }

    #[test]
    fn integrated_backend_rejects_commitment_witness_mismatch() {
        let polynomial_under_test = polynomial(&[2, 3, 5, 7]);
        let other_polynomial = polynomial(&[2, 3, 5, 8]);
        let point_coordinates = vec![Fr::from(3), Fr::from(3)];
        let (prover_key, _verifier_key) = keys(2);

        let bad_witness = commit_ipa_backend(&prover_key, &other_polynomial, Fr::from(9)).unwrap();
        let mut transcript = MerlinTranscript::new(b"ipa-integrated-backend-test");

        assert!(matches!(
            open_ipa_backend(
                &prover_key,
                &bad_witness,
                &polynomial_under_test,
                &point_coordinates,
                &mut transcript,
            ),
            Err(IpaBackendIntegrationError::BlindedPath(
                IpaBlindedPathError::CommitmentWitnessMismatch
            ))
        ));
    }

    #[test]
    fn integrated_backend_rejects_tampered_opening() {
        let polynomial = polynomial(&[2, 3, 5, 7]);
        let point_coordinates = vec![Fr::from(3), Fr::from(3)];
        let (prover_key, verifier_key) = keys(2);

        let witness = commit_ipa_backend(&prover_key, &polynomial, Fr::from(9)).unwrap();

        let mut prover_transcript = MerlinTranscript::new(b"ipa-integrated-backend-test");
        let mut opening = open_ipa_backend(
            &prover_key,
            &witness,
            &polynomial,
            &point_coordinates,
            &mut prover_transcript,
        )
        .unwrap();

        opening.proof.final_polynomial_scalar += Fr::from(1);

        let mut verifier_transcript = MerlinTranscript::new(b"ipa-integrated-backend-test");
        assert!(verify_ipa_backend(
            &verifier_key,
            &witness.commitment,
            &point_coordinates,
            &opening,
            &mut verifier_transcript,
        )
        .is_err());
    }

    #[test]
    fn integrated_keys_reject_bad_padding_shape() {
        assert_eq!(
            trim_ipa_integrated_keys(
                basis(2),
                point(5000),
                vec![point(2000)],
                padding_evaluation(2),
                point(9000),
            ),
            Err(IpaBackendIntegrationError::PaddingGeneratorCountMismatch {
                label: "polynomial",
                expected: 3,
                actual: 1,
            })
        );
    }
}
