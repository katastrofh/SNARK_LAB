use ark_ec::{AffineRepr, CurveGroup};
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize, Compress, Validate};
use multilinear::Multilinear;

use crate::ipa::IpaCommitment;
use crate::ipa_curve::{IpaCurveGeneratorBasis, IpaCurvePointError};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IpaCommitmentEquationError {
    Curve(IpaCurvePointError),
    VariableCountMismatch {
        polynomial_variables: usize,
        basis_variables: usize,
    },
    GeneratorCountMismatch {
        expected: usize,
        actual: usize,
    },
    SerializationFailed,
    DeserializationFailed,
}

impl From<IpaCurvePointError> for IpaCommitmentEquationError {
    fn from(error: IpaCurvePointError) -> Self {
        Self::Curve(error)
    }
}

/// Curve commitment produced by the IPA commitment equation.
///
/// Unlike generator points, a commitment is allowed to be the identity because
/// committing to the zero vector with zero blinding is mathematically valid.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IpaCurveCommitment<G: CurveGroup> {
    affine: G::Affine,
}

impl<G> IpaCurveCommitment<G>
where
    G: CurveGroup,
    G::Affine: AffineRepr + CanonicalSerialize + CanonicalDeserialize,
{
    pub fn from_projective(point: G) -> Result<Self, IpaCommitmentEquationError> {
        Ok(Self {
            affine: point.into_affine(),
        })
    }

    pub fn from_compressed_bytes(bytes: &[u8]) -> Result<Self, IpaCommitmentEquationError> {
        let affine = G::Affine::deserialize_with_mode(bytes, Compress::Yes, Validate::Yes)
            .map_err(|_| IpaCommitmentEquationError::DeserializationFailed)?;

        Ok(Self { affine })
    }

    pub fn affine(&self) -> G::Affine {
        self.affine
    }

    pub fn to_compressed_bytes(&self) -> Result<Vec<u8>, IpaCommitmentEquationError> {
        let mut out = Vec::new();

        self.affine
            .serialize_with_mode(&mut out, Compress::Yes)
            .map_err(|_| IpaCommitmentEquationError::SerializationFailed)?;

        Ok(out)
    }

    pub fn to_opaque_commitment(
        &self,
        variables: usize,
    ) -> Result<IpaCommitment, IpaCommitmentEquationError> {
        Ok(IpaCommitment {
            variables,
            commitment_bytes: self.to_compressed_bytes()?,
        })
    }
}

/// Validate that the commitment equation has compatible dimensions.
pub fn validate_ipa_commitment_inputs<G>(
    basis: &IpaCurveGeneratorBasis<G>,
    polynomial: &Multilinear<G::ScalarField>,
) -> Result<(), IpaCommitmentEquationError>
where
    G: CurveGroup,
    G::Affine: AffineRepr + CanonicalSerialize + CanonicalDeserialize,
{
    if polynomial.variables() != basis.variables {
        return Err(IpaCommitmentEquationError::VariableCountMismatch {
            polynomial_variables: polynomial.variables(),
            basis_variables: basis.variables,
        });
    }

    basis.validate()?;

    let expected = polynomial.evaluations().len();
    let actual = basis.polynomial_generators.len();

    if expected != actual {
        return Err(IpaCommitmentEquationError::GeneratorCountMismatch { expected, actual });
    }

    Ok(())
}

/// Compute the IPA commitment equation:
///
/// ```text
/// C = <a, G> + rH
/// ```
///
/// where:
///
/// - `a` is the multilinear evaluation vector,
/// - `G` is the polynomial-generator basis,
/// - `r` is the blinding scalar,
/// - `H` is the blinding generator.
pub fn commit_ipa_polynomial<G>(
    basis: &IpaCurveGeneratorBasis<G>,
    polynomial: &Multilinear<G::ScalarField>,
    blinding: G::ScalarField,
) -> Result<IpaCurveCommitment<G>, IpaCommitmentEquationError>
where
    G: CurveGroup,
    G::Affine: AffineRepr + CanonicalSerialize + CanonicalDeserialize,
{
    validate_ipa_commitment_inputs(basis, polynomial)?;

    let mut accumulator = G::zero();

    for (scalar, generator) in polynomial
        .evaluations()
        .iter()
        .zip(basis.polynomial_generators.iter())
    {
        accumulator += generator.affine().into_group() * *scalar;
    }

    accumulator += basis.blinding_generator.affine().into_group() * blinding;

    IpaCurveCommitment::from_projective(accumulator)
}

/// Recompute and compare the commitment equation.
///
/// This is a witness-side consistency check. It is not an IPA opening verifier.
pub fn check_ipa_commitment_equation<G>(
    commitment: &IpaCurveCommitment<G>,
    basis: &IpaCurveGeneratorBasis<G>,
    polynomial: &Multilinear<G::ScalarField>,
    blinding: G::ScalarField,
) -> Result<bool, IpaCommitmentEquationError>
where
    G: CurveGroup,
    G::Affine: AffineRepr + CanonicalSerialize + CanonicalDeserialize,
{
    let recomputed = commit_ipa_polynomial(basis, polynomial, blinding)?;

    Ok(&recomputed == commitment)
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

    fn polynomial(values: &[u64]) -> Multilinear<Fr> {
        Multilinear::new(values.iter().copied().map(Fr::from).collect()).unwrap()
    }

    fn commitment_group(commitment: &IpaCurveCommitment<G1Projective>) -> G1Projective {
        commitment.affine().into_group()
    }

    #[test]
    fn commitment_equation_matches_manual_linear_combination() {
        let basis = basis(2);
        let polynomial = polynomial(&[3, 5, 7, 11]);
        let blinding = Fr::from(13);

        let commitment = commit_ipa_polynomial(&basis, &polynomial, blinding).unwrap();

        let mut manual = G1Projective::zero();

        for (scalar, generator) in polynomial
            .evaluations()
            .iter()
            .zip(basis.polynomial_generators.iter())
        {
            manual += generator.affine().into_group() * *scalar;
        }

        manual += basis.blinding_generator.affine().into_group() * blinding;

        let manual_commitment = IpaCurveCommitment::from_projective(manual).unwrap();

        assert_eq!(commitment, manual_commitment);
    }

    #[test]
    fn commitment_equation_check_accepts_matching_commitment() {
        let basis = basis(2);
        let polynomial = polynomial(&[3, 5, 7, 11]);
        let blinding = Fr::from(13);

        let commitment = commit_ipa_polynomial(&basis, &polynomial, blinding).unwrap();

        assert_eq!(
            check_ipa_commitment_equation(&commitment, &basis, &polynomial, blinding),
            Ok(true)
        );
    }

    #[test]
    fn commitment_equation_check_rejects_wrong_blinding() {
        let basis = basis(2);
        let polynomial = polynomial(&[3, 5, 7, 11]);

        let commitment = commit_ipa_polynomial(&basis, &polynomial, Fr::from(13)).unwrap();

        assert_eq!(
            check_ipa_commitment_equation(&commitment, &basis, &polynomial, Fr::from(14)),
            Ok(false)
        );
    }

    #[test]
    fn commitment_equation_rejects_variable_mismatch() {
        let basis = basis(1);
        let polynomial = polynomial(&[3, 5, 7, 11]);

        assert_eq!(
            commit_ipa_polynomial(&basis, &polynomial, Fr::from(13)),
            Err(IpaCommitmentEquationError::VariableCountMismatch {
                polynomial_variables: 2,
                basis_variables: 1
            })
        );
    }

    #[test]
    fn commitment_equation_rejects_generator_count_mismatch() {
        let mut basis = basis(2);
        basis.polynomial_generators.pop();

        let polynomial = polynomial(&[3, 5, 7, 11]);

        assert_eq!(
            commit_ipa_polynomial(&basis, &polynomial, Fr::from(13)),
            Err(IpaCommitmentEquationError::Curve(
                IpaCurvePointError::InvalidGeneratorCount {
                    label: "polynomial",
                    expected: 4,
                    actual: 3
                }
            ))
        );
    }

    #[test]
    fn commitment_equation_is_linear_in_polynomial_and_blinding() {
        let basis = basis(1);
        let p1 = polynomial(&[2, 3]);
        let p2 = polynomial(&[5, 7]);
        let p_sum = polynomial(&[7, 10]);

        let c1 = commit_ipa_polynomial(&basis, &p1, Fr::from(11)).unwrap();
        let c2 = commit_ipa_polynomial(&basis, &p2, Fr::from(13)).unwrap();
        let c_sum = commit_ipa_polynomial(&basis, &p_sum, Fr::from(24)).unwrap();

        let combined =
            IpaCurveCommitment::from_projective(commitment_group(&c1) + commitment_group(&c2))
                .unwrap();

        assert_eq!(combined, c_sum);
    }

    #[test]
    fn curve_commitment_roundtrip_uses_canonical_compressed_bytes() {
        let basis = basis(2);
        let polynomial = polynomial(&[3, 5, 7, 11]);
        let commitment = commit_ipa_polynomial(&basis, &polynomial, Fr::from(13)).unwrap();

        let encoded = commitment.to_compressed_bytes().unwrap();
        let decoded = IpaCurveCommitment::<G1Projective>::from_compressed_bytes(&encoded).unwrap();

        assert_eq!(commitment, decoded);
        assert_eq!(encoded, decoded.to_compressed_bytes().unwrap());
    }

    #[test]
    fn curve_commitment_converts_to_opaque_commitment() {
        let basis = basis(1);
        let polynomial = polynomial(&[3, 5]);
        let commitment = commit_ipa_polynomial(&basis, &polynomial, Fr::from(13)).unwrap();

        let opaque = commitment
            .to_opaque_commitment(polynomial.variables())
            .unwrap();

        assert_eq!(opaque.variables, 1);
        assert_eq!(
            opaque.commitment_bytes,
            commitment.to_compressed_bytes().unwrap()
        );
    }
}
