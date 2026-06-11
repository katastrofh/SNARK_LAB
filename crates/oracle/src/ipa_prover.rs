use ark_ec::{AffineRepr, CurveGroup};
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use multilinear::Multilinear;

use crate::ipa::IpaCommitment;
use crate::ipa_commitment::{
    commit_ipa_polynomial, IpaCommitmentEquationError, IpaCurveCommitment,
};
use crate::ipa_curve::{IpaCurveGeneratorBasis, IpaCurvePointError};
use crate::pcs::{validate_supported_variables, PcsShapeError};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IpaProverCommitError {
    Shape(PcsShapeError),
    Curve(IpaCurvePointError),
    Commitment(IpaCommitmentEquationError),
}

impl From<PcsShapeError> for IpaProverCommitError {
    fn from(error: PcsShapeError) -> Self {
        Self::Shape(error)
    }
}

impl From<IpaCurvePointError> for IpaProverCommitError {
    fn from(error: IpaCurvePointError) -> Self {
        Self::Curve(error)
    }
}

impl From<IpaCommitmentEquationError> for IpaProverCommitError {
    fn from(error: IpaCommitmentEquationError) -> Self {
        Self::Commitment(error)
    }
}

/// Curve-aware prover key for the IPA commit path.
///
/// This is separate from the earlier shape-only `IpaProverKey`. It carries the
/// actual generator basis required to compute:
///
/// ```text
/// C = <a, G> + rH
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IpaCurveProverKey<G: CurveGroup> {
    pub supported_variables: usize,
    pub generator_basis: IpaCurveGeneratorBasis<G>,
}

impl<G> IpaCurveProverKey<G>
where
    G: CurveGroup,
    G::Affine: AffineRepr + CanonicalSerialize + CanonicalDeserialize,
{
    pub fn new(
        supported_variables: usize,
        generator_basis: IpaCurveGeneratorBasis<G>,
    ) -> Result<Self, IpaProverCommitError> {
        validate_supported_variables(supported_variables, generator_basis.variables)?;
        generator_basis.validate()?;

        Ok(Self {
            supported_variables,
            generator_basis,
        })
    }

    pub fn variables(&self) -> usize {
        self.generator_basis.variables
    }
}

/// Result of the prover commit path.
///
/// The curve commitment is kept for later prover-side IPA opening work. The
/// opaque commitment is the protocol-facing commitment bytes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IpaProverCommitment<G: CurveGroup> {
    pub curve_commitment: IpaCurveCommitment<G>,
    pub opaque_commitment: IpaCommitment,
}

/// Commit a multilinear polynomial with an explicit blinding scalar.
///
/// This is a real curve commitment computation. It is not yet a complete IPA
/// proof system because it does not produce an opening proof.
pub fn commit_with_ipa_prover_key<G>(
    prover_key: &IpaCurveProverKey<G>,
    polynomial: &Multilinear<G::ScalarField>,
    blinding: G::ScalarField,
) -> Result<IpaProverCommitment<G>, IpaProverCommitError>
where
    G: CurveGroup,
    G::Affine: AffineRepr + CanonicalSerialize + CanonicalDeserialize,
{
    validate_supported_variables(prover_key.supported_variables, polynomial.variables())?;

    let curve_commitment =
        commit_ipa_polynomial(&prover_key.generator_basis, polynomial, blinding)?;

    let opaque_commitment = curve_commitment.to_opaque_commitment(polynomial.variables())?;

    Ok(IpaProverCommitment {
        curve_commitment,
        opaque_commitment,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use ark_bls12_381::{Fr, G1Projective};
    use ark_ec::PrimeGroup;

    use crate::ipa_commitment::check_ipa_commitment_equation;
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

    fn polynomial(values: &[u64]) -> Multilinear<Fr> {
        Multilinear::new(values.iter().copied().map(Fr::from).collect()).unwrap()
    }

    #[test]
    fn prover_key_accepts_basis_within_supported_variables() {
        let key = IpaCurveProverKey::new(4, basis(2)).unwrap();

        assert_eq!(key.supported_variables, 4);
        assert_eq!(key.variables(), 2);
    }

    #[test]
    fn prover_key_rejects_basis_above_supported_variables() {
        assert_eq!(
            IpaCurveProverKey::new(1, basis(2)),
            Err(IpaProverCommitError::Shape(
                PcsShapeError::UnsupportedVariableCount {
                    requested: 2,
                    supported: 1
                }
            ))
        );
    }

    #[test]
    fn prover_commit_path_returns_curve_and_opaque_commitments() {
        let key = IpaCurveProverKey::new(2, basis(2)).unwrap();
        let polynomial = polynomial(&[3, 5, 7, 11]);
        let blinding = Fr::from(13);

        let committed = commit_with_ipa_prover_key(&key, &polynomial, blinding).unwrap();

        assert_eq!(committed.opaque_commitment.variables, 2);
        assert_eq!(
            committed.opaque_commitment.commitment_bytes,
            committed.curve_commitment.to_compressed_bytes().unwrap()
        );

        assert_eq!(
            check_ipa_commitment_equation(
                &committed.curve_commitment,
                &key.generator_basis,
                &polynomial,
                blinding
            ),
            Ok(true)
        );
    }

    #[test]
    fn prover_commit_path_rejects_polynomial_above_supported_variables() {
        let key = IpaCurveProverKey::new(1, basis(1)).unwrap();
        let polynomial = polynomial(&[3, 5, 7, 11]);

        assert_eq!(
            commit_with_ipa_prover_key(&key, &polynomial, Fr::from(13)),
            Err(IpaProverCommitError::Shape(
                PcsShapeError::UnsupportedVariableCount {
                    requested: 2,
                    supported: 1
                }
            ))
        );
    }

    #[test]
    fn prover_commit_path_rejects_basis_polynomial_mismatch() {
        let key = IpaCurveProverKey::new(2, basis(1)).unwrap();
        let polynomial = polynomial(&[3, 5, 7, 11]);

        assert_eq!(
            commit_with_ipa_prover_key(&key, &polynomial, Fr::from(13)),
            Err(IpaProverCommitError::Commitment(
                IpaCommitmentEquationError::VariableCountMismatch {
                    polynomial_variables: 2,
                    basis_variables: 1
                }
            ))
        );
    }

    #[test]
    fn prover_commit_path_is_deterministic_for_same_blinding() {
        let key = IpaCurveProverKey::new(2, basis(2)).unwrap();
        let polynomial = polynomial(&[3, 5, 7, 11]);

        let a = commit_with_ipa_prover_key(&key, &polynomial, Fr::from(13)).unwrap();
        let b = commit_with_ipa_prover_key(&key, &polynomial, Fr::from(13)).unwrap();

        assert_eq!(a, b);
    }

    #[test]
    fn prover_commit_path_changes_when_blinding_changes() {
        let key = IpaCurveProverKey::new(2, basis(2)).unwrap();
        let polynomial = polynomial(&[3, 5, 7, 11]);

        let a = commit_with_ipa_prover_key(&key, &polynomial, Fr::from(13)).unwrap();
        let b = commit_with_ipa_prover_key(&key, &polynomial, Fr::from(14)).unwrap();

        assert_ne!(a, b);
    }
}
