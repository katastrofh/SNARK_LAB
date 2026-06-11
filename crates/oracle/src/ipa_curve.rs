use std::collections::HashSet;

use ark_ec::{AffineRepr, CurveGroup};
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize, Compress, Validate};
use snark_lab_transcript::ProofTranscript;

use crate::ipa_generators::{
    bind_ipa_generator_basis, expected_ipa_generator_count, IpaGeneratorBasis,
    IpaGeneratorBasisError,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IpaCurvePointError {
    SerializationFailed,
    DeserializationFailed,
    IdentityPoint,
    DuplicatePoint {
        label: &'static str,
        index: usize,
    },
    InvalidGeneratorCount {
        label: &'static str,
        expected: usize,
        actual: usize,
    },
    InvalidOpaqueBasis(IpaGeneratorBasisError),
}

/// Canonically serialized IPA curve point.
///
/// The wrapped affine point is guaranteed to be non-identity when constructed
/// through this API. Deserialization uses arkworks validation, which includes
/// canonical encoding and subgroup validation for the selected curve type.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IpaCurvePoint<G: CurveGroup> {
    affine: G::Affine,
}

impl<G> IpaCurvePoint<G>
where
    G: CurveGroup,
    G::Affine: AffineRepr + CanonicalSerialize + CanonicalDeserialize,
{
    pub fn from_projective(point: G) -> Result<Self, IpaCurvePointError> {
        Self::from_affine(point.into_affine())
    }

    pub fn from_affine(affine: G::Affine) -> Result<Self, IpaCurvePointError> {
        if affine.is_zero() {
            return Err(IpaCurvePointError::IdentityPoint);
        }

        Ok(Self { affine })
    }

    pub fn from_compressed_bytes(bytes: &[u8]) -> Result<Self, IpaCurvePointError> {
        let affine = G::Affine::deserialize_with_mode(bytes, Compress::Yes, Validate::Yes)
            .map_err(|_| IpaCurvePointError::DeserializationFailed)?;

        Self::from_affine(affine)
    }

    pub fn affine(&self) -> G::Affine {
        self.affine
    }

    pub fn to_compressed_bytes(&self) -> Result<Vec<u8>, IpaCurvePointError> {
        let mut out = Vec::new();

        self.affine
            .serialize_with_mode(&mut out, Compress::Yes)
            .map_err(|_| IpaCurvePointError::SerializationFailed)?;

        Ok(out)
    }
}

/// Typed IPA generator basis over a concrete curve group.
///
/// This is the first curve-aware layer. It replaces opaque byte vectors with
/// checked curve points, but still converts back to the existing byte-level
/// `IpaGeneratorBasis` for transcript binding and compatibility.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IpaCurveGeneratorBasis<G: CurveGroup> {
    pub variables: usize,
    pub polynomial_generators: Vec<IpaCurvePoint<G>>,
    pub evaluation_generators: Vec<IpaCurvePoint<G>>,
    pub blinding_generator: IpaCurvePoint<G>,
}

impl<G> IpaCurveGeneratorBasis<G>
where
    G: CurveGroup,
    G::Affine: AffineRepr + CanonicalSerialize + CanonicalDeserialize,
{
    pub fn new(
        variables: usize,
        polynomial_generators: Vec<IpaCurvePoint<G>>,
        evaluation_generators: Vec<IpaCurvePoint<G>>,
        blinding_generator: IpaCurvePoint<G>,
    ) -> Result<Self, IpaCurvePointError> {
        let basis = Self {
            variables,
            polynomial_generators,
            evaluation_generators,
            blinding_generator,
        };

        basis.validate()?;

        Ok(basis)
    }

    pub fn validate(&self) -> Result<(), IpaCurvePointError> {
        let expected = expected_ipa_generator_count(self.variables)
            .map_err(IpaCurvePointError::InvalidOpaqueBasis)?;

        validate_curve_generator_collection("polynomial", expected, &self.polynomial_generators)?;
        validate_curve_generator_collection("evaluation", expected, &self.evaluation_generators)?;

        let mut seen = HashSet::new();

        insert_generator_bytes("polynomial", &self.polynomial_generators, &mut seen)?;
        insert_generator_bytes("evaluation", &self.evaluation_generators, &mut seen)?;

        let blinding = self.blinding_generator.to_compressed_bytes()?;
        if !seen.insert(blinding) {
            return Err(IpaCurvePointError::DuplicatePoint {
                label: "blinding",
                index: 0,
            });
        }

        Ok(())
    }

    pub fn to_opaque_basis(&self) -> Result<IpaGeneratorBasis, IpaCurvePointError> {
        self.validate()?;

        let polynomial_generators = self
            .polynomial_generators
            .iter()
            .map(IpaCurvePoint::to_compressed_bytes)
            .collect::<Result<Vec<_>, _>>()?;

        let evaluation_generators = self
            .evaluation_generators
            .iter()
            .map(IpaCurvePoint::to_compressed_bytes)
            .collect::<Result<Vec<_>, _>>()?;

        let blinding_generator = self.blinding_generator.to_compressed_bytes()?;

        IpaGeneratorBasis::new(
            self.variables,
            polynomial_generators,
            evaluation_generators,
            blinding_generator,
        )
        .map_err(IpaCurvePointError::InvalidOpaqueBasis)
    }
}

fn validate_curve_generator_collection<G>(
    label: &'static str,
    expected: usize,
    generators: &[IpaCurvePoint<G>],
) -> Result<(), IpaCurvePointError>
where
    G: CurveGroup,
    G::Affine: AffineRepr + CanonicalSerialize + CanonicalDeserialize,
{
    if generators.len() != expected {
        return Err(IpaCurvePointError::InvalidGeneratorCount {
            label,
            expected,
            actual: generators.len(),
        });
    }

    Ok(())
}

fn insert_generator_bytes<G>(
    label: &'static str,
    generators: &[IpaCurvePoint<G>],
    seen: &mut HashSet<Vec<u8>>,
) -> Result<(), IpaCurvePointError>
where
    G: CurveGroup,
    G::Affine: AffineRepr + CanonicalSerialize + CanonicalDeserialize,
{
    for (index, generator) in generators.iter().enumerate() {
        let bytes = generator.to_compressed_bytes()?;

        if !seen.insert(bytes) {
            return Err(IpaCurvePointError::DuplicatePoint { label, index });
        }
    }

    Ok(())
}

pub fn bind_ipa_curve_generator_basis<G, T>(
    transcript: &mut T,
    basis: &IpaCurveGeneratorBasis<G>,
) -> Result<(), IpaCurvePointError>
where
    G: CurveGroup,
    G::Affine: AffineRepr + CanonicalSerialize + CanonicalDeserialize,
    T: ProofTranscript<G::ScalarField>,
{
    let opaque = basis.to_opaque_basis()?;

    bind_ipa_generator_basis::<G::ScalarField, _>(transcript, &opaque)
        .map_err(IpaCurvePointError::InvalidOpaqueBasis)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ark_bls12_381::{Fr, G1Projective};
    use ark_ec::PrimeGroup;
    use ark_ff::Zero;
    use snark_lab_transcript::{MerlinTranscript, ProofTranscript};

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

    fn bound_challenge(basis: &IpaCurveGeneratorBasis<G1Projective>) -> Fr {
        let mut transcript = MerlinTranscript::new(b"ipa-curve-generator-test");

        bind_ipa_curve_generator_basis(&mut transcript, basis).unwrap();

        transcript.challenge_scalar(b"after-curve-generators")
    }

    #[test]
    fn curve_point_roundtrip_uses_compressed_bytes() {
        let original = point(42);
        let bytes = original.to_compressed_bytes().unwrap();
        let decoded = IpaCurvePoint::<G1Projective>::from_compressed_bytes(&bytes).unwrap();

        assert_eq!(original, decoded);
        assert_eq!(bytes, decoded.to_compressed_bytes().unwrap());
    }

    #[test]
    fn curve_point_rejects_identity() {
        assert_eq!(
            IpaCurvePoint::<G1Projective>::from_projective(G1Projective::zero()),
            Err(IpaCurvePointError::IdentityPoint)
        );
    }

    #[test]
    fn curve_point_rejects_invalid_bytes() {
        assert_eq!(
            IpaCurvePoint::<G1Projective>::from_compressed_bytes(&[1, 2, 3]),
            Err(IpaCurvePointError::DeserializationFailed)
        );
    }

    #[test]
    fn curve_generator_basis_accepts_valid_points() {
        let basis = basis(2);

        assert_eq!(basis.variables, 2);
        assert_eq!(basis.polynomial_generators.len(), 4);
        assert_eq!(basis.evaluation_generators.len(), 4);
        assert_eq!(basis.validate(), Ok(()));
    }

    #[test]
    fn curve_generator_basis_rejects_wrong_count() {
        let mut polynomial = (0..3).map(|index| point(index + 1)).collect::<Vec<_>>();
        polynomial.pop();

        assert_eq!(
            IpaCurveGeneratorBasis::new(
                2,
                polynomial,
                (0..4).map(|index| point(index + 100)).collect(),
                point(999)
            ),
            Err(IpaCurvePointError::InvalidGeneratorCount {
                label: "polynomial",
                expected: 4,
                actual: 2
            })
        );
    }

    #[test]
    fn curve_generator_basis_rejects_duplicates_across_sets() {
        let duplicate = point(3);
        let polynomial = vec![point(1), duplicate.clone()];
        let evaluation = vec![duplicate, point(100)];

        assert_eq!(
            IpaCurveGeneratorBasis::new(1, polynomial, evaluation, point(999)),
            Err(IpaCurvePointError::DuplicatePoint {
                label: "evaluation",
                index: 0
            })
        );
    }

    #[test]
    fn curve_generator_basis_converts_to_opaque_basis() {
        let basis = basis(1);
        let opaque = basis.to_opaque_basis().unwrap();

        assert_eq!(opaque.variables, 1);
        assert_eq!(opaque.polynomial_generators.len(), 2);
        assert_eq!(opaque.evaluation_generators.len(), 2);
        assert!(opaque.validate().is_ok());
    }

    #[test]
    fn curve_generator_basis_binding_is_deterministic() {
        let basis = basis(2);

        assert_eq!(bound_challenge(&basis), bound_challenge(&basis));
    }

    #[test]
    fn curve_generator_basis_binding_changes_when_point_changes() {
        let a = basis(2);
        let mut b = basis(2);

        b.polynomial_generators[0] = point(777);

        assert_ne!(bound_challenge(&a), bound_challenge(&b));
    }
}
