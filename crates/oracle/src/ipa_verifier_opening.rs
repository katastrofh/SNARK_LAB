use ark_ec::{AffineRepr, CurveGroup};
use ark_ff::{Field, PrimeField};
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use snark_lab_transcript::ProofTranscript;

use crate::ipa_commitment::{IpaCommitmentEquationError, IpaCurveCommitment};
use crate::ipa_curve::{IpaCurveGeneratorBasis, IpaCurvePoint, IpaCurvePointError};
use crate::ipa_generator_folding::{fold_ipa_generator_basis, IpaGeneratorFoldingError};
use crate::ipa_opening_statement::{
    bind_ipa_opening_statement_context, validate_ipa_opening_statement, IpaOpeningStatement,
    IpaOpeningStatementError,
};
use crate::ipa_proof::{validate_ipa_opening_proof_shape, IpaOpeningProof, IpaProofShapeError};
use crate::ipa_reduction::{IpaReductionRound, IpaReductionRoundError};
use crate::ipa_transcript::IpaTranscriptRound;

const IPA_PROVER_OPENING_ROUND_DOMAIN: &[u8] = b"snark-lab/ipa-prover-opening-round/v1";

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IpaVerifierOpeningError<F: PrimeField> {
    Curve(IpaCurvePointError),
    Commitment(IpaCommitmentEquationError),
    GeneratorFolding(IpaGeneratorFoldingError<F>),
    OpeningStatement(IpaOpeningStatementError<F>),
    ProofShape(IpaProofShapeError),
    Reduction(IpaReductionRoundError<F>),
    BasisVariableMismatch {
        basis_variables: usize,
        statement_variables: usize,
    },
    ProofVariableMismatch {
        proof_variables: usize,
        statement_variables: usize,
    },
    ProofClaimMismatch {
        proof_claimed: F,
        statement_claimed: F,
    },
    GeneratorCountMismatch {
        expected: usize,
        actual: usize,
    },
    UnexpectedRoundIndex {
        expected: usize,
        actual: usize,
    },
    FinalCommitmentMismatch,
}

impl<F: PrimeField> From<IpaCurvePointError> for IpaVerifierOpeningError<F> {
    fn from(error: IpaCurvePointError) -> Self {
        Self::Curve(error)
    }
}

impl<F: PrimeField> From<IpaCommitmentEquationError> for IpaVerifierOpeningError<F> {
    fn from(error: IpaCommitmentEquationError) -> Self {
        Self::Commitment(error)
    }
}

impl<F: PrimeField> From<IpaGeneratorFoldingError<F>> for IpaVerifierOpeningError<F> {
    fn from(error: IpaGeneratorFoldingError<F>) -> Self {
        Self::GeneratorFolding(error)
    }
}

impl<F: PrimeField> From<IpaOpeningStatementError<F>> for IpaVerifierOpeningError<F> {
    fn from(error: IpaOpeningStatementError<F>) -> Self {
        Self::OpeningStatement(error)
    }
}

impl<F: PrimeField> From<IpaProofShapeError> for IpaVerifierOpeningError<F> {
    fn from(error: IpaProofShapeError) -> Self {
        Self::ProofShape(error)
    }
}

impl<F: PrimeField> From<IpaReductionRoundError<F>> for IpaVerifierOpeningError<F> {
    fn from(error: IpaReductionRoundError<F>) -> Self {
        Self::Reduction(error)
    }
}

fn derive_round_challenge<F: PrimeField, T: ProofTranscript<F>>(
    transcript: &mut T,
    round_index: usize,
    input_length: usize,
    transcript_round: &IpaTranscriptRound,
) -> F {
    transcript.append_domain_separator(IPA_PROVER_OPENING_ROUND_DOMAIN);
    transcript.append_field_modulus();
    transcript.append_u64(b"ipa-prover-opening-round-index", round_index as u64);
    transcript.append_u64(b"ipa-prover-opening-input-length", input_length as u64);
    transcript.append_bytes(
        b"ipa-prover-opening-left-commitment",
        &transcript_round.left_commitment_bytes,
    );
    transcript.append_bytes(
        b"ipa-prover-opening-right-commitment",
        &transcript_round.right_commitment_bytes,
    );

    transcript.challenge_scalar(b"ipa-prover-opening-challenge")
}

fn linear_combination<G>(
    scalars: &[G::ScalarField],
    generators: &[IpaCurvePoint<G>],
) -> Result<G, IpaVerifierOpeningError<G::ScalarField>>
where
    G: CurveGroup,
    G::Affine: AffineRepr + CanonicalSerialize + CanonicalDeserialize,
{
    if scalars.len() != generators.len() {
        return Err(IpaVerifierOpeningError::GeneratorCountMismatch {
            expected: scalars.len(),
            actual: generators.len(),
        });
    }

    let mut accumulator = G::zero();

    for (scalar, generator) in scalars.iter().zip(generators.iter()) {
        accumulator += generator.affine().into_group() * *scalar;
    }

    Ok(accumulator)
}

fn initial_relation_commitment<G>(
    basis: &IpaCurveGeneratorBasis<G>,
    statement: &IpaOpeningStatement<G::ScalarField>,
    inner_product_generator: &IpaCurvePoint<G>,
) -> Result<IpaCurveCommitment<G>, IpaVerifierOpeningError<G::ScalarField>>
where
    G: CurveGroup,
    G::Affine: AffineRepr + CanonicalSerialize + CanonicalDeserialize,
{
    basis.validate()?;

    if basis.variables != statement.variables() {
        return Err(IpaVerifierOpeningError::BasisVariableMismatch {
            basis_variables: basis.variables,
            statement_variables: statement.variables(),
        });
    }

    let mut relation =
        IpaCurveCommitment::<G>::from_compressed_bytes(&statement.commitment.commitment_bytes)?
            .affine()
            .into_group();

    relation += linear_combination::<G>(
        &statement.evaluation_basis.basis_evaluations,
        &basis.evaluation_generators,
    )?;

    relation += inner_product_generator.affine().into_group() * statement.claimed_value;

    Ok(IpaCurveCommitment::from_projective(relation)?)
}

fn update_relation_commitment<G>(
    current: &IpaCurveCommitment<G>,
    left: &IpaCurveCommitment<G>,
    right: &IpaCurveCommitment<G>,
    challenge: G::ScalarField,
) -> Result<IpaCurveCommitment<G>, IpaVerifierOpeningError<G::ScalarField>>
where
    G: CurveGroup,
    G::Affine: AffineRepr + CanonicalSerialize + CanonicalDeserialize,
{
    let challenge_inverse = challenge
        .inverse()
        .ok_or(IpaVerifierOpeningError::Reduction(
            IpaReductionRoundError::ZeroChallenge,
        ))?;

    let challenge_squared = challenge * challenge;
    let challenge_inverse_squared = challenge_inverse * challenge_inverse;

    let updated = current.affine().into_group()
        + left.affine().into_group() * challenge_squared
        + right.affine().into_group() * challenge_inverse_squared;

    Ok(IpaCurveCommitment::from_projective(updated)?)
}

fn final_relation_commitment<G>(
    basis: &IpaCurveGeneratorBasis<G>,
    final_polynomial_scalar: G::ScalarField,
    final_evaluation_basis_scalar: G::ScalarField,
    inner_product_generator: &IpaCurvePoint<G>,
) -> Result<IpaCurveCommitment<G>, IpaVerifierOpeningError<G::ScalarField>>
where
    G: CurveGroup,
    G::Affine: AffineRepr + CanonicalSerialize + CanonicalDeserialize,
{
    basis.validate()?;

    if basis.polynomial_generators.len() != 1 {
        return Err(IpaVerifierOpeningError::GeneratorCountMismatch {
            expected: 1,
            actual: basis.polynomial_generators.len(),
        });
    }

    if basis.evaluation_generators.len() != 1 {
        return Err(IpaVerifierOpeningError::GeneratorCountMismatch {
            expected: 1,
            actual: basis.evaluation_generators.len(),
        });
    }

    let product = final_polynomial_scalar * final_evaluation_basis_scalar;

    let relation = basis.polynomial_generators[0].affine().into_group() * final_polynomial_scalar
        + basis.evaluation_generators[0].affine().into_group() * final_evaluation_basis_scalar
        + inner_product_generator.affine().into_group() * product;

    Ok(IpaCurveCommitment::from_projective(relation)?)
}

/// Verify an IPA opening proof against a public opening statement.
///
/// This is a real verifier-side recursive check for the currently supported
/// unblinded IPA relation. It rejects malformed proofs and non-matching final
/// relations.
///
/// Hiding/blinding support is intentionally not accepted here yet; nonzero
/// blinded commitments should fail until the blinding-opening extension exists.
pub fn verify_ipa_opening<G, T>(
    basis: &IpaCurveGeneratorBasis<G>,
    statement: &IpaOpeningStatement<G::ScalarField>,
    proof: &IpaOpeningProof<G::ScalarField>,
    inner_product_generator: &IpaCurvePoint<G>,
    transcript: &mut T,
) -> Result<(), IpaVerifierOpeningError<G::ScalarField>>
where
    G: CurveGroup,
    G::Affine: AffineRepr + CanonicalSerialize + CanonicalDeserialize,
    T: ProofTranscript<G::ScalarField>,
{
    validate_ipa_opening_statement(statement)?;
    validate_ipa_opening_proof_shape(proof)?;

    if proof.variables != statement.variables() {
        return Err(IpaVerifierOpeningError::ProofVariableMismatch {
            proof_variables: proof.variables,
            statement_variables: statement.variables(),
        });
    }

    if proof.claimed_value != statement.claimed_value {
        return Err(IpaVerifierOpeningError::ProofClaimMismatch {
            proof_claimed: proof.claimed_value,
            statement_claimed: statement.claimed_value,
        });
    }

    bind_ipa_opening_statement_context(transcript, statement)?;

    let mut current_basis = basis.clone();
    let mut current_commitment =
        initial_relation_commitment(&current_basis, statement, inner_product_generator)?;

    let mut current_length = statement.evaluation_basis.basis_evaluations.len();

    for (expected_round_index, round) in proof.rounds.iter().enumerate() {
        if round.round_index != expected_round_index {
            return Err(IpaVerifierOpeningError::UnexpectedRoundIndex {
                expected: expected_round_index,
                actual: round.round_index,
            });
        }

        let challenge =
            derive_round_challenge(transcript, expected_round_index, current_length, round);

        IpaReductionRound::new(
            expected_round_index,
            round.left_commitment_bytes.clone(),
            round.right_commitment_bytes.clone(),
            challenge,
            current_length,
        )?;

        let left_commitment =
            IpaCurveCommitment::<G>::from_compressed_bytes(&round.left_commitment_bytes)?;
        let right_commitment =
            IpaCurveCommitment::<G>::from_compressed_bytes(&round.right_commitment_bytes)?;

        current_commitment = update_relation_commitment(
            &current_commitment,
            &left_commitment,
            &right_commitment,
            challenge,
        )?;

        current_basis = fold_ipa_generator_basis(&current_basis, challenge)?;
        current_length /= 2;
    }

    let expected_final_relation = final_relation_commitment(
        &current_basis,
        proof.final_polynomial_scalar,
        proof.final_evaluation_basis_scalar,
        inner_product_generator,
    )?;

    let proof_final_relation =
        IpaCurveCommitment::<G>::from_compressed_bytes(&proof.final_commitment_bytes)?;

    if current_commitment.affine().into_group() != proof_final_relation.affine().into_group() {
        return Err(IpaVerifierOpeningError::FinalCommitmentMismatch);
    }

    if expected_final_relation.affine().into_group() != proof_final_relation.affine().into_group() {
        return Err(IpaVerifierOpeningError::FinalCommitmentMismatch);
    }

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
    use crate::ipa_opening_statement::IpaOpeningStatement;
    use crate::ipa_prover_opening::{prove_ipa_opening, IpaProverOpeningOutput};

    fn point_generator(seed: u64) -> IpaCurvePoint<G1Projective> {
        IpaCurvePoint::from_projective(G1Projective::generator() * Fr::from(seed)).unwrap()
    }

    fn basis(variables: usize) -> IpaCurveGeneratorBasis<G1Projective> {
        let count = expected_ipa_generator_count(variables).unwrap();

        IpaCurveGeneratorBasis::new(
            variables,
            (0..count)
                .map(|index| point_generator(index as u64 + 1))
                .collect(),
            (0..count)
                .map(|index| point_generator(index as u64 + 100))
                .collect(),
            point_generator(999),
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
    ) -> crate::ipa::IpaCommitment {
        commit_ipa_polynomial(basis, polynomial, blinding)
            .unwrap()
            .to_opaque_commitment(polynomial.variables())
            .unwrap()
    }

    fn prover_output(
        values: &[u64],
        blinding: Fr,
    ) -> (
        IpaCurveGeneratorBasis<G1Projective>,
        IpaCurvePoint<G1Projective>,
        IpaProverOpeningOutput<G1Projective>,
    ) {
        let polynomial = polynomial(values);
        let basis = basis(polynomial.variables());
        let commitment = commitment_for(&basis, &polynomial, blinding);
        let point = vec![Fr::from(3); polynomial.variables()];
        let claimed = polynomial.evaluate(&point).unwrap();
        let inner_product_generator = point_generator(5000);
        let mut transcript = MerlinTranscript::new(b"ipa-verifier-opening-test");

        let output = prove_ipa_opening(
            &basis,
            commitment,
            &polynomial,
            &point,
            claimed,
            &inner_product_generator,
            &mut transcript,
        )
        .unwrap();

        (basis, inner_product_generator, output)
    }

    fn verify_output(
        basis: &IpaCurveGeneratorBasis<G1Projective>,
        inner_product_generator: &IpaCurvePoint<G1Projective>,
        output: &IpaProverOpeningOutput<G1Projective>,
    ) -> Result<(), IpaVerifierOpeningError<Fr>> {
        let mut transcript = MerlinTranscript::new(b"ipa-verifier-opening-test");

        verify_ipa_opening(
            basis,
            &output.statement,
            &output.proof,
            inner_product_generator,
            &mut transcript,
        )
    }

    #[test]
    fn verifier_accepts_prover_opening_output() {
        let (basis, inner_product_generator, output) =
            prover_output(&[2, 3, 5, 7, 11, 13, 17, 19], Fr::from(0));

        assert_eq!(
            verify_output(&basis, &inner_product_generator, &output),
            Ok(())
        );
    }

    #[test]
    fn verifier_accepts_zero_variable_opening() {
        let (basis, inner_product_generator, output) = prover_output(&[42], Fr::from(0));

        assert_eq!(
            verify_output(&basis, &inner_product_generator, &output),
            Ok(())
        );
    }

    #[test]
    fn verifier_rejects_proof_claim_mismatch() {
        let (basis, inner_product_generator, mut output) =
            prover_output(&[2, 3, 5, 7], Fr::from(0));

        output.proof.claimed_value += Fr::from(1);

        assert!(matches!(
            verify_output(&basis, &inner_product_generator, &output),
            Err(IpaVerifierOpeningError::ProofClaimMismatch { .. })
        ));
    }

    #[test]
    fn verifier_rejects_tampered_final_scalar() {
        let (basis, inner_product_generator, mut output) =
            prover_output(&[2, 3, 5, 7], Fr::from(0));

        output.proof.final_polynomial_scalar += Fr::from(1);

        assert_eq!(
            verify_output(&basis, &inner_product_generator, &output),
            Err(IpaVerifierOpeningError::FinalCommitmentMismatch)
        );
    }

    #[test]
    fn verifier_rejects_tampered_round_commitment() {
        let (basis, inner_product_generator, mut output) =
            prover_output(&[2, 3, 5, 7], Fr::from(0));

        output.proof.rounds[0].left_commitment_bytes =
            output.proof.rounds[0].right_commitment_bytes.clone();

        assert!(matches!(
            verify_output(&basis, &inner_product_generator, &output),
            Err(IpaVerifierOpeningError::FinalCommitmentMismatch)
                | Err(IpaVerifierOpeningError::Curve(_))
        ));
    }

    #[test]
    fn verifier_rejects_wrong_round_index() {
        let (basis, inner_product_generator, mut output) =
            prover_output(&[2, 3, 5, 7], Fr::from(0));

        output.proof.rounds[0].round_index = 7;

        assert_eq!(
            verify_output(&basis, &inner_product_generator, &output),
            Err(IpaVerifierOpeningError::UnexpectedRoundIndex {
                expected: 0,
                actual: 7
            })
        );
    }

    #[test]
    fn verifier_rejects_wrong_public_commitment() {
        let (basis, inner_product_generator, mut output) =
            prover_output(&[2, 3, 5, 7], Fr::from(0));

        let other_polynomial = polynomial(&[2, 3, 5, 8]);
        output.statement = IpaOpeningStatement::new(
            commitment_for(&basis, &other_polynomial, Fr::from(0)),
            output.statement.point.clone(),
            output.statement.claimed_value,
        )
        .unwrap();

        assert_eq!(
            verify_output(&basis, &inner_product_generator, &output),
            Err(IpaVerifierOpeningError::FinalCommitmentMismatch)
        );
    }

    #[test]
    fn verifier_rejects_blinded_commitment_until_blinding_extension_exists() {
        let (basis, inner_product_generator, output) = prover_output(&[2, 3, 5, 7], Fr::from(9));

        assert_eq!(
            verify_output(&basis, &inner_product_generator, &output),
            Err(IpaVerifierOpeningError::FinalCommitmentMismatch)
        );
    }
}
