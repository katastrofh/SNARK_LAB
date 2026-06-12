use ark_ff::PrimeField;
use multilinear::Multilinear;
use snark_lab_transcript::ProofTranscript;

use crate::ipa::IpaCommitment;
use crate::ipa_evaluation::{
    bind_ipa_evaluation_basis, compute_ipa_evaluation_basis, IpaEvaluationBasis,
    IpaEvaluationBasisError,
};

const IPA_OPENING_STATEMENT_DOMAIN: &[u8] = b"snark-lab/ipa-opening-statement/v1";

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IpaOpeningStatementError<F: PrimeField> {
    EvaluationBasis(IpaEvaluationBasisError),
    CommitmentVariableMismatch {
        commitment_variables: usize,
        point_variables: usize,
    },
    EvaluationBasisVariableMismatch {
        basis_variables: usize,
        point_variables: usize,
    },
    EvaluationBasisPointMismatch,
    EvaluationBasisLengthMismatch {
        expected: usize,
        actual: usize,
    },
    PolynomialVariableMismatch {
        polynomial_variables: usize,
        statement_variables: usize,
    },
    ClaimedValueMismatch {
        claimed: F,
        computed: F,
    },
}

impl<F: PrimeField> From<IpaEvaluationBasisError> for IpaOpeningStatementError<F> {
    fn from(error: IpaEvaluationBasisError) -> Self {
        Self::EvaluationBasis(error)
    }
}

/// Full public statement for an IPA opening claim.
///
/// The statement binds:
///
/// ```text
/// commitment C
/// opening point z
/// claimed value v
/// evaluation basis eq(z, ·)
/// ```
///
/// This is not yet an IPA verifier. It is the verifier-side statement object
/// that future IPA reduction-round checks will consume.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IpaOpeningStatement<F: PrimeField> {
    pub commitment: IpaCommitment,
    pub point: Vec<F>,
    pub claimed_value: F,
    pub evaluation_basis: IpaEvaluationBasis<F>,
}

impl<F: PrimeField> IpaOpeningStatement<F> {
    pub fn new(
        commitment: IpaCommitment,
        point: Vec<F>,
        claimed_value: F,
    ) -> Result<Self, IpaOpeningStatementError<F>> {
        let evaluation_basis = compute_ipa_evaluation_basis(&point)?;

        Self::from_parts(commitment, point, claimed_value, evaluation_basis)
    }

    pub fn from_parts(
        commitment: IpaCommitment,
        point: Vec<F>,
        claimed_value: F,
        evaluation_basis: IpaEvaluationBasis<F>,
    ) -> Result<Self, IpaOpeningStatementError<F>> {
        if commitment.variables != point.len() {
            return Err(IpaOpeningStatementError::CommitmentVariableMismatch {
                commitment_variables: commitment.variables,
                point_variables: point.len(),
            });
        }

        if evaluation_basis.variables != point.len() {
            return Err(IpaOpeningStatementError::EvaluationBasisVariableMismatch {
                basis_variables: evaluation_basis.variables,
                point_variables: point.len(),
            });
        }

        if evaluation_basis.point != point {
            return Err(IpaOpeningStatementError::EvaluationBasisPointMismatch);
        }

        let expected = evaluation_basis.expected_len()?;
        let actual = evaluation_basis.basis_evaluations.len();

        if actual != expected {
            return Err(IpaOpeningStatementError::EvaluationBasisLengthMismatch {
                expected,
                actual,
            });
        }

        Ok(Self {
            commitment,
            point,
            claimed_value,
            evaluation_basis,
        })
    }

    pub fn variables(&self) -> usize {
        self.point.len()
    }
}

/// Build an IPA opening statement and check that the claimed value is consistent
/// with a known polynomial witness.
///
/// This is a prover-side / test-side helper. A public verifier must not use this
/// with a hidden polynomial witness.
pub fn opening_statement_from_witness<F: PrimeField>(
    commitment: IpaCommitment,
    polynomial: &Multilinear<F>,
    point: &[F],
    claimed_value: F,
) -> Result<IpaOpeningStatement<F>, IpaOpeningStatementError<F>> {
    if polynomial.variables() != point.len() {
        return Err(IpaOpeningStatementError::PolynomialVariableMismatch {
            polynomial_variables: polynomial.variables(),
            statement_variables: point.len(),
        });
    }

    let statement = IpaOpeningStatement::new(commitment, point.to_vec(), claimed_value)?;
    let computed = statement.evaluation_basis.evaluate_polynomial(polynomial)?;

    if computed != claimed_value {
        return Err(IpaOpeningStatementError::ClaimedValueMismatch {
            claimed: claimed_value,
            computed,
        });
    }

    Ok(statement)
}

/// Check only the public statement shape.
///
/// This does not verify the IPA proof. It only checks dimensional consistency
/// between commitment, point, claimed value, and evaluation basis.
pub fn validate_ipa_opening_statement<F: PrimeField>(
    statement: &IpaOpeningStatement<F>,
) -> Result<(), IpaOpeningStatementError<F>> {
    IpaOpeningStatement::from_parts(
        statement.commitment.clone(),
        statement.point.clone(),
        statement.claimed_value,
        statement.evaluation_basis.clone(),
    )
    .map(|_| ())
}

/// Bind the full opening statement into the Fiat-Shamir transcript.
pub fn bind_ipa_opening_statement_context<F: PrimeField, T: ProofTranscript<F>>(
    transcript: &mut T,
    statement: &IpaOpeningStatement<F>,
) -> Result<(), IpaOpeningStatementError<F>> {
    validate_ipa_opening_statement(statement)?;

    transcript.append_domain_separator(IPA_OPENING_STATEMENT_DOMAIN);
    transcript.append_field_modulus();
    transcript.append_u64(b"ipa-opening-variables", statement.variables() as u64);
    transcript.append_u64(
        b"ipa-opening-commitment-variables",
        statement.commitment.variables as u64,
    );
    transcript.append_bytes(
        b"ipa-opening-commitment",
        &statement.commitment.commitment_bytes,
    );

    for (index, coordinate) in statement.point.iter().enumerate() {
        transcript.append_u64(b"ipa-opening-point-index", index as u64);
        transcript.append_field_element(b"ipa-opening-point-coordinate", coordinate);
    }

    transcript.append_field_element(b"ipa-opening-claimed-value", &statement.claimed_value);

    bind_ipa_evaluation_basis(transcript, &statement.evaluation_basis)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ark_bls12_381::Fr;
    use snark_lab_transcript::{MerlinTranscript, ProofTranscript};

    fn commitment(variables: usize, seed: u8) -> IpaCommitment {
        IpaCommitment {
            variables,
            commitment_bytes: vec![seed, seed + 1, seed + 2],
        }
    }

    fn polynomial(values: &[u64]) -> Multilinear<Fr> {
        Multilinear::new(values.iter().copied().map(Fr::from).collect()).unwrap()
    }

    fn challenge_for_statement(statement: &IpaOpeningStatement<Fr>) -> Fr {
        let mut transcript = MerlinTranscript::new(b"ipa-opening-statement-test");

        bind_ipa_opening_statement_context::<Fr, _>(&mut transcript, statement).unwrap();

        transcript.challenge_scalar(b"after-opening-statement")
    }

    #[test]
    fn opening_statement_accepts_consistent_shape() {
        let statement = IpaOpeningStatement::new(
            commitment(2, 7),
            vec![Fr::from(3), Fr::from(5)],
            Fr::from(99),
        )
        .unwrap();

        assert_eq!(statement.variables(), 2);
        assert_eq!(statement.commitment.variables, 2);
        assert_eq!(statement.evaluation_basis.variables, 2);
        assert_eq!(validate_ipa_opening_statement(&statement), Ok(()));
    }

    #[test]
    fn opening_statement_rejects_commitment_variable_mismatch() {
        assert_eq!(
            IpaOpeningStatement::new(
                commitment(1, 7),
                vec![Fr::from(3), Fr::from(5)],
                Fr::from(99),
            ),
            Err(IpaOpeningStatementError::CommitmentVariableMismatch {
                commitment_variables: 1,
                point_variables: 2,
            })
        );
    }

    #[test]
    fn opening_statement_rejects_evaluation_basis_variable_mismatch() {
        let basis = compute_ipa_evaluation_basis::<Fr>(&[Fr::from(3)]).unwrap();

        assert_eq!(
            IpaOpeningStatement::from_parts(
                commitment(2, 7),
                vec![Fr::from(3), Fr::from(5)],
                Fr::from(99),
                basis,
            ),
            Err(IpaOpeningStatementError::EvaluationBasisVariableMismatch {
                basis_variables: 1,
                point_variables: 2,
            })
        );
    }

    #[test]
    fn opening_statement_rejects_evaluation_basis_point_mismatch() {
        let basis = compute_ipa_evaluation_basis::<Fr>(&[Fr::from(3), Fr::from(6)]).unwrap();

        assert_eq!(
            IpaOpeningStatement::from_parts(
                commitment(2, 7),
                vec![Fr::from(3), Fr::from(5)],
                Fr::from(99),
                basis,
            ),
            Err(IpaOpeningStatementError::EvaluationBasisPointMismatch)
        );
    }

    #[test]
    fn opening_statement_from_witness_accepts_correct_claim() {
        let polynomial = polynomial(&[2, 3, 5, 7]);
        let point = [Fr::from(3), Fr::from(5)];
        let claimed = polynomial.evaluate(&point).unwrap();

        let statement =
            opening_statement_from_witness(commitment(2, 7), &polynomial, &point, claimed).unwrap();

        assert_eq!(statement.claimed_value, claimed);
    }

    #[test]
    fn opening_statement_from_witness_rejects_wrong_claim() {
        let polynomial = polynomial(&[2, 3, 5, 7]);
        let point = [Fr::from(3), Fr::from(5)];
        let computed = polynomial.evaluate(&point).unwrap();
        let wrong = computed + Fr::from(1);

        assert_eq!(
            opening_statement_from_witness(commitment(2, 7), &polynomial, &point, wrong),
            Err(IpaOpeningStatementError::ClaimedValueMismatch {
                claimed: wrong,
                computed,
            })
        );
    }

    #[test]
    fn opening_statement_from_witness_rejects_polynomial_variable_mismatch() {
        let polynomial = polynomial(&[2, 3, 5, 7]);

        assert_eq!(
            opening_statement_from_witness(
                commitment(1, 7),
                &polynomial,
                &[Fr::from(3)],
                Fr::from(99),
            ),
            Err(IpaOpeningStatementError::PolynomialVariableMismatch {
                polynomial_variables: 2,
                statement_variables: 1,
            })
        );
    }

    #[test]
    fn opening_statement_binding_is_deterministic() {
        let statement = IpaOpeningStatement::new(
            commitment(2, 7),
            vec![Fr::from(3), Fr::from(5)],
            Fr::from(99),
        )
        .unwrap();

        assert_eq!(
            challenge_for_statement(&statement),
            challenge_for_statement(&statement)
        );
    }

    #[test]
    fn opening_statement_binding_changes_when_commitment_changes() {
        let a = IpaOpeningStatement::new(
            commitment(2, 7),
            vec![Fr::from(3), Fr::from(5)],
            Fr::from(99),
        )
        .unwrap();

        let b = IpaOpeningStatement::new(
            commitment(2, 8),
            vec![Fr::from(3), Fr::from(5)],
            Fr::from(99),
        )
        .unwrap();

        assert_ne!(challenge_for_statement(&a), challenge_for_statement(&b));
    }

    #[test]
    fn opening_statement_binding_changes_when_claim_changes() {
        let a = IpaOpeningStatement::new(
            commitment(2, 7),
            vec![Fr::from(3), Fr::from(5)],
            Fr::from(99),
        )
        .unwrap();

        let b = IpaOpeningStatement::new(
            commitment(2, 7),
            vec![Fr::from(3), Fr::from(5)],
            Fr::from(100),
        )
        .unwrap();

        assert_ne!(challenge_for_statement(&a), challenge_for_statement(&b));
    }

    #[test]
    fn zero_variable_opening_statement_is_valid() {
        let statement =
            IpaOpeningStatement::new(commitment(0, 7), Vec::<Fr>::new(), Fr::from(99)).unwrap();

        assert_eq!(statement.variables(), 0);
        assert_eq!(statement.evaluation_basis.basis_evaluations.len(), 1);
    }
}
