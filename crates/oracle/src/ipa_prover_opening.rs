use ark_ec::{AffineRepr, CurveGroup};
use ark_ff::PrimeField;
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use multilinear::Multilinear;
use snark_lab_transcript::ProofTranscript;

use crate::ipa::IpaCommitment;
use crate::ipa_commitment::{IpaCommitmentEquationError, IpaCurveCommitment};
use crate::ipa_curve::{IpaCurveGeneratorBasis, IpaCurvePoint, IpaCurvePointError};
use crate::ipa_evaluation::IpaEvaluationBasisError;
use crate::ipa_generator_folding::{fold_ipa_generator_basis, IpaGeneratorFoldingError};
use crate::ipa_opening_statement::{
    bind_ipa_opening_statement_context, opening_statement_from_witness, IpaOpeningStatement,
    IpaOpeningStatementError,
};
use crate::ipa_proof::{IpaOpeningProof, IpaProofShapeError};
use crate::ipa_reduction::{
    fold_ipa_evaluation_vector, fold_ipa_polynomial_vector, IpaReductionRoundError,
};
use crate::ipa_round_commitments::{
    compute_ipa_round_commitments, IpaRoundCommitmentError, IpaRoundCommitments,
};
use crate::ipa_transcript::IpaTranscriptRound;

const IPA_PROVER_OPENING_ROUND_DOMAIN: &[u8] = b"snark-lab/ipa-prover-opening-round/v1";

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IpaProverOpeningError<F: PrimeField> {
    Curve(IpaCurvePointError),
    Commitment(IpaCommitmentEquationError),
    EvaluationBasis(IpaEvaluationBasisError),
    OpeningStatement(IpaOpeningStatementError<F>),
    ProofShape(IpaProofShapeError),
    Reduction(IpaReductionRoundError<F>),
    RoundCommitment(IpaRoundCommitmentError<F>),
    GeneratorFolding(IpaGeneratorFoldingError<F>),
    EmptyFinalVector,
    FinalGeneratorCountMismatch {
        polynomial_generators: usize,
        evaluation_generators: usize,
    },
}

impl<F: PrimeField> From<IpaCurvePointError> for IpaProverOpeningError<F> {
    fn from(error: IpaCurvePointError) -> Self {
        Self::Curve(error)
    }
}

impl<F: PrimeField> From<IpaCommitmentEquationError> for IpaProverOpeningError<F> {
    fn from(error: IpaCommitmentEquationError) -> Self {
        Self::Commitment(error)
    }
}

impl<F: PrimeField> From<IpaEvaluationBasisError> for IpaProverOpeningError<F> {
    fn from(error: IpaEvaluationBasisError) -> Self {
        Self::EvaluationBasis(error)
    }
}

impl<F: PrimeField> From<IpaOpeningStatementError<F>> for IpaProverOpeningError<F> {
    fn from(error: IpaOpeningStatementError<F>) -> Self {
        Self::OpeningStatement(error)
    }
}

impl<F: PrimeField> From<IpaProofShapeError> for IpaProverOpeningError<F> {
    fn from(error: IpaProofShapeError) -> Self {
        Self::ProofShape(error)
    }
}

impl<F: PrimeField> From<IpaReductionRoundError<F>> for IpaProverOpeningError<F> {
    fn from(error: IpaReductionRoundError<F>) -> Self {
        Self::Reduction(error)
    }
}

impl<F: PrimeField> From<IpaRoundCommitmentError<F>> for IpaProverOpeningError<F> {
    fn from(error: IpaRoundCommitmentError<F>) -> Self {
        Self::RoundCommitment(error)
    }
}

impl<F: PrimeField> From<IpaGeneratorFoldingError<F>> for IpaProverOpeningError<F> {
    fn from(error: IpaGeneratorFoldingError<F>) -> Self {
        Self::GeneratorFolding(error)
    }
}

/// Prover-side output for the IPA opening loop.
///
/// `proof` is the public proof-shaped object. The remaining fields are typed
/// prover-side artifacts useful for internal tests and future verifier
/// integration. They should not be treated as a verifier acceptance signal.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IpaProverOpeningOutput<G: CurveGroup> {
    pub statement: IpaOpeningStatement<G::ScalarField>,
    pub proof: IpaOpeningProof<G::ScalarField>,
    pub round_commitments: Vec<IpaRoundCommitments<G>>,
    pub final_generator_basis: IpaCurveGeneratorBasis<G>,
    pub final_relation_commitment: IpaCurveCommitment<G>,
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

fn final_relation_commitment<G>(
    basis: &IpaCurveGeneratorBasis<G>,
    final_polynomial_scalar: G::ScalarField,
    final_evaluation_scalar: G::ScalarField,
    inner_product_generator: &IpaCurvePoint<G>,
) -> Result<IpaCurveCommitment<G>, IpaProverOpeningError<G::ScalarField>>
where
    G: CurveGroup,
    G::Affine: AffineRepr + CanonicalSerialize + CanonicalDeserialize,
{
    basis.validate()?;

    if basis.polynomial_generators.len() != 1 || basis.evaluation_generators.len() != 1 {
        return Err(IpaProverOpeningError::FinalGeneratorCountMismatch {
            polynomial_generators: basis.polynomial_generators.len(),
            evaluation_generators: basis.evaluation_generators.len(),
        });
    }

    let product = final_polynomial_scalar * final_evaluation_scalar;

    let relation = basis.polynomial_generators[0].affine().into_group() * final_polynomial_scalar
        + basis.evaluation_generators[0].affine().into_group() * final_evaluation_scalar
        + inner_product_generator.affine().into_group() * product;

    Ok(IpaCurveCommitment::from_projective(relation)?)
}

/// Generate the prover-side IPA opening loop for a committed multilinear table.
///
/// This function performs real algebraic proof construction steps:
///
/// - binds the public opening statement,
/// - computes every `L` and `R` round commitment,
/// - derives Fiat-Shamir challenges,
/// - folds polynomial vector, evaluation vector, and generator basis,
/// - emits final scalars and final relation commitment bytes.
///
/// This is not a verifier. It never returns an acceptance decision.
pub fn prove_ipa_opening<G, T>(
    basis: &IpaCurveGeneratorBasis<G>,
    commitment: IpaCommitment,
    polynomial: &Multilinear<G::ScalarField>,
    point: &[G::ScalarField],
    claimed_value: G::ScalarField,
    inner_product_generator: &IpaCurvePoint<G>,
    transcript: &mut T,
) -> Result<IpaProverOpeningOutput<G>, IpaProverOpeningError<G::ScalarField>>
where
    G: CurveGroup,
    G::Affine: AffineRepr + CanonicalSerialize + CanonicalDeserialize,
    T: ProofTranscript<G::ScalarField>,
{
    let statement = opening_statement_from_witness(commitment, polynomial, point, claimed_value)?;

    bind_ipa_opening_statement_context(transcript, &statement)?;

    basis.validate()?;

    let mut current_polynomial_vector = polynomial.evaluations().to_vec();
    let mut current_evaluation_vector = statement.evaluation_basis.basis_evaluations.clone();
    let mut current_basis = basis.clone();

    let mut transcript_rounds = Vec::new();
    let mut round_commitments = Vec::new();

    while current_polynomial_vector.len() > 1 {
        let round_index = transcript_rounds.len();

        let commitments = compute_ipa_round_commitments(
            round_index,
            &current_basis,
            &current_polynomial_vector,
            &current_evaluation_vector,
            inner_product_generator,
        )?;

        let transcript_round = commitments.to_transcript_round()?;
        let challenge = derive_round_challenge(
            transcript,
            round_index,
            current_polynomial_vector.len(),
            &transcript_round,
        );

        commitments.to_reduction_round(challenge)?;

        current_polynomial_vector =
            fold_ipa_polynomial_vector(&current_polynomial_vector, challenge)?;
        current_evaluation_vector =
            fold_ipa_evaluation_vector(&current_evaluation_vector, challenge)?;
        current_basis = fold_ipa_generator_basis(&current_basis, challenge)?;

        transcript_rounds.push(transcript_round);
        round_commitments.push(commitments);
    }

    let final_polynomial_scalar = *current_polynomial_vector
        .first()
        .ok_or(IpaProverOpeningError::EmptyFinalVector)?;
    let final_evaluation_basis_scalar = *current_evaluation_vector
        .first()
        .ok_or(IpaProverOpeningError::EmptyFinalVector)?;

    let final_relation_commitment = final_relation_commitment(
        &current_basis,
        final_polynomial_scalar,
        final_evaluation_basis_scalar,
        inner_product_generator,
    )?;

    let final_commitment_bytes = final_relation_commitment.to_compressed_bytes()?;

    let proof = IpaOpeningProof::new(
        statement.variables(),
        statement.claimed_value,
        transcript_rounds,
        final_polynomial_scalar,
        final_evaluation_basis_scalar,
        final_commitment_bytes,
    )?;

    Ok(IpaProverOpeningOutput {
        statement,
        proof,
        round_commitments,
        final_generator_basis: current_basis,
        final_relation_commitment,
    })
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
    ) -> IpaCommitment {
        commit_ipa_polynomial(basis, polynomial, Fr::from(0))
            .unwrap()
            .to_opaque_commitment(polynomial.variables())
            .unwrap()
    }

    fn prove_for_values(values: &[u64]) -> IpaProverOpeningOutput<G1Projective> {
        let polynomial = polynomial(values);
        let basis = basis(polynomial.variables());
        let commitment = commitment_for(&basis, &polynomial);
        let point = vec![Fr::from(3); polynomial.variables()];
        let claimed = polynomial.evaluate(&point).unwrap();
        let inner_product_generator = point_generator(5000);
        let mut transcript = MerlinTranscript::new(b"ipa-prover-opening-test");

        prove_ipa_opening(
            &basis,
            commitment,
            &polynomial,
            &point,
            claimed,
            &inner_product_generator,
            &mut transcript,
        )
        .unwrap()
    }

    #[test]
    fn prover_opening_loop_produces_expected_round_count() {
        let output = prove_for_values(&[2, 3, 5, 7, 11, 13, 17, 19]);

        assert_eq!(output.statement.variables(), 3);
        assert_eq!(output.proof.rounds.len(), 3);
        assert_eq!(output.round_commitments.len(), 3);
        assert_eq!(output.final_generator_basis.variables, 0);
        assert_eq!(output.final_generator_basis.polynomial_generators.len(), 1);
        assert_eq!(output.final_generator_basis.evaluation_generators.len(), 1);
        assert!(!output.proof.final_commitment_bytes.is_empty());
    }

    #[test]
    fn prover_opening_loop_is_deterministic_for_same_inputs() {
        let first = prove_for_values(&[2, 3, 5, 7]);
        let second = prove_for_values(&[2, 3, 5, 7]);

        assert_eq!(first.proof, second.proof);
        assert_eq!(first.final_generator_basis, second.final_generator_basis);
        assert_eq!(
            first.final_relation_commitment,
            second.final_relation_commitment
        );
    }

    #[test]
    fn prover_opening_loop_changes_when_polynomial_changes() {
        let first = prove_for_values(&[2, 3, 5, 7]);
        let second = prove_for_values(&[2, 3, 5, 8]);

        assert_ne!(first.proof, second.proof);
    }

    #[test]
    fn prover_opening_loop_rejects_wrong_claim() {
        let polynomial = polynomial(&[2, 3, 5, 7]);
        let basis = basis(2);
        let commitment = commitment_for(&basis, &polynomial);
        let point = vec![Fr::from(3), Fr::from(5)];
        let inner_product_generator = point_generator(5000);
        let mut transcript = MerlinTranscript::new(b"ipa-prover-opening-test");

        assert!(matches!(
            prove_ipa_opening(
                &basis,
                commitment,
                &polynomial,
                &point,
                Fr::from(123),
                &inner_product_generator,
                &mut transcript,
            ),
            Err(IpaProverOpeningError::OpeningStatement(
                IpaOpeningStatementError::ClaimedValueMismatch { .. }
            ))
        ));
    }

    #[test]
    fn prover_opening_loop_rejects_basis_variable_mismatch() {
        let polynomial = polynomial(&[2, 3, 5, 7]);
        let bad_basis = basis(1);
        let good_basis = basis(2);
        let commitment = commitment_for(&good_basis, &polynomial);
        let point = vec![Fr::from(3), Fr::from(5)];
        let claimed = polynomial.evaluate(&point).unwrap();
        let inner_product_generator = point_generator(5000);
        let mut transcript = MerlinTranscript::new(b"ipa-prover-opening-test");

        assert!(matches!(
            prove_ipa_opening(
                &bad_basis,
                commitment,
                &polynomial,
                &point,
                claimed,
                &inner_product_generator,
                &mut transcript,
            ),
            Err(IpaProverOpeningError::RoundCommitment(
                IpaRoundCommitmentError::GeneratorCountMismatch { .. }
            ))
        ));
    }

    #[test]
    fn prover_opening_loop_handles_zero_variable_polynomial() {
        let output = prove_for_values(&[42]);

        assert_eq!(output.statement.variables(), 0);
        assert_eq!(output.proof.rounds.len(), 0);
        assert_eq!(output.round_commitments.len(), 0);
        assert_eq!(output.final_generator_basis.variables, 0);
        assert!(!output.proof.final_commitment_bytes.is_empty());
    }
}
