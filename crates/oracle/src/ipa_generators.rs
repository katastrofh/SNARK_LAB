use std::collections::HashSet;

use ark_ff::PrimeField;
use snark_lab_transcript::ProofTranscript;

const IPA_GENERATOR_BASIS_DOMAIN: &[u8] = b"snark-lab/ipa-generator-basis/v1";

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IpaGeneratorBasisError {
    VariableCountOverflow {
        variables: usize,
    },
    InvalidGeneratorCount {
        label: &'static str,
        expected: usize,
        actual: usize,
    },
    EmptyGenerator {
        label: &'static str,
        index: usize,
    },
    AllZeroGenerator {
        label: &'static str,
        index: usize,
    },
    DuplicateGenerator {
        label: &'static str,
        index: usize,
    },
}

/// Opaque generator basis for an IPA-style multilinear PCS.
///
/// This module fixes the basis boundary before group arithmetic is introduced.
/// The byte vectors are placeholders for future canonical group-element encodings,
/// not proof bytes and not verifier accept conditions.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IpaGeneratorBasis {
    pub variables: usize,
    pub polynomial_generators: Vec<Vec<u8>>,
    pub evaluation_generators: Vec<Vec<u8>>,
    pub blinding_generator: Vec<u8>,
}

impl IpaGeneratorBasis {
    pub fn new(
        variables: usize,
        polynomial_generators: Vec<Vec<u8>>,
        evaluation_generators: Vec<Vec<u8>>,
        blinding_generator: Vec<u8>,
    ) -> Result<Self, IpaGeneratorBasisError> {
        let basis = Self {
            variables,
            polynomial_generators,
            evaluation_generators,
            blinding_generator,
        };

        basis.validate()?;

        Ok(basis)
    }

    pub fn generator_count(&self) -> Result<usize, IpaGeneratorBasisError> {
        expected_ipa_generator_count(self.variables)
    }

    pub fn validate(&self) -> Result<(), IpaGeneratorBasisError> {
        let expected = expected_ipa_generator_count(self.variables)?;
        let mut seen = HashSet::new();

        validate_generator_collection(
            "polynomial",
            expected,
            &self.polynomial_generators,
            &mut seen,
        )?;

        validate_generator_collection(
            "evaluation",
            expected,
            &self.evaluation_generators,
            &mut seen,
        )?;

        validate_single_generator("blinding", 0, &self.blinding_generator, &mut seen)?;

        Ok(())
    }
}

/// A multilinear table over `variables` variables has `2^variables` entries.
pub fn expected_ipa_generator_count(variables: usize) -> Result<usize, IpaGeneratorBasisError> {
    let shift = u32::try_from(variables)
        .map_err(|_| IpaGeneratorBasisError::VariableCountOverflow { variables })?;

    1usize
        .checked_shl(shift)
        .ok_or(IpaGeneratorBasisError::VariableCountOverflow { variables })
}

fn validate_generator_collection(
    label: &'static str,
    expected: usize,
    generators: &[Vec<u8>],
    seen: &mut HashSet<Vec<u8>>,
) -> Result<(), IpaGeneratorBasisError> {
    if generators.len() != expected {
        return Err(IpaGeneratorBasisError::InvalidGeneratorCount {
            label,
            expected,
            actual: generators.len(),
        });
    }

    for (index, generator) in generators.iter().enumerate() {
        validate_single_generator(label, index, generator, seen)?;
    }

    Ok(())
}

fn validate_single_generator(
    label: &'static str,
    index: usize,
    generator: &[u8],
    seen: &mut HashSet<Vec<u8>>,
) -> Result<(), IpaGeneratorBasisError> {
    if generator.is_empty() {
        return Err(IpaGeneratorBasisError::EmptyGenerator { label, index });
    }

    if generator.iter().all(|byte| *byte == 0) {
        return Err(IpaGeneratorBasisError::AllZeroGenerator { label, index });
    }

    if !seen.insert(generator.to_vec()) {
        return Err(IpaGeneratorBasisError::DuplicateGenerator { label, index });
    }

    Ok(())
}

/// Bind the full IPA generator basis into the Fiat-Shamir transcript.
///
/// This makes verifier challenges depend on the exact generator material.
pub fn bind_ipa_generator_basis<F: PrimeField, T: ProofTranscript<F>>(
    transcript: &mut T,
    basis: &IpaGeneratorBasis,
) -> Result<(), IpaGeneratorBasisError> {
    basis.validate()?;

    transcript.append_domain_separator(IPA_GENERATOR_BASIS_DOMAIN);
    transcript.append_field_modulus();
    transcript.append_u64(b"ipa-generator-variables", basis.variables as u64);
    transcript.append_u64(b"ipa-generator-count", basis.generator_count()? as u64);

    for (index, generator) in basis.polynomial_generators.iter().enumerate() {
        transcript.append_u64(b"ipa-polynomial-generator-index", index as u64);
        transcript.append_bytes(b"ipa-polynomial-generator", generator);
    }

    for (index, generator) in basis.evaluation_generators.iter().enumerate() {
        transcript.append_u64(b"ipa-evaluation-generator-index", index as u64);
        transcript.append_bytes(b"ipa-evaluation-generator", generator);
    }

    transcript.append_bytes(b"ipa-blinding-generator", &basis.blinding_generator);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ark_bls12_381::Fr;
    use snark_lab_transcript::{MerlinTranscript, ProofTranscript};

    fn generator(prefix: u8, index: usize) -> Vec<u8> {
        vec![prefix, index as u8 + 1, index as u8 + 17]
    }

    fn basis(variables: usize) -> IpaGeneratorBasis {
        let count = expected_ipa_generator_count(variables).unwrap();

        IpaGeneratorBasis::new(
            variables,
            (0..count).map(|index| generator(1, index)).collect(),
            (0..count).map(|index| generator(2, index)).collect(),
            vec![9, 9, 9],
        )
        .unwrap()
    }

    fn bound_challenge(basis: &IpaGeneratorBasis) -> Fr {
        let mut transcript = MerlinTranscript::new(b"ipa-generator-basis-test");

        bind_ipa_generator_basis::<Fr, _>(&mut transcript, basis).unwrap();

        transcript.challenge_scalar(b"after-generator-basis")
    }

    #[test]
    fn expected_generator_count_is_power_of_two() {
        assert_eq!(expected_ipa_generator_count(0), Ok(1));
        assert_eq!(expected_ipa_generator_count(1), Ok(2));
        assert_eq!(expected_ipa_generator_count(4), Ok(16));
    }

    #[test]
    fn expected_generator_count_rejects_overflow() {
        assert_eq!(
            expected_ipa_generator_count(usize::BITS as usize),
            Err(IpaGeneratorBasisError::VariableCountOverflow {
                variables: usize::BITS as usize
            })
        );
    }

    #[test]
    fn generator_basis_accepts_valid_shape() {
        let basis = basis(3);

        assert_eq!(basis.variables, 3);
        assert_eq!(basis.generator_count(), Ok(8));
        assert_eq!(basis.validate(), Ok(()));
    }

    #[test]
    fn generator_basis_rejects_wrong_polynomial_count() {
        let count = expected_ipa_generator_count(2).unwrap();

        assert_eq!(
            IpaGeneratorBasis::new(
                2,
                (0..count - 1).map(|index| generator(1, index)).collect(),
                (0..count).map(|index| generator(2, index)).collect(),
                vec![9],
            ),
            Err(IpaGeneratorBasisError::InvalidGeneratorCount {
                label: "polynomial",
                expected: 4,
                actual: 3
            })
        );
    }

    #[test]
    fn generator_basis_rejects_empty_generator() {
        let count = expected_ipa_generator_count(1).unwrap();
        let mut polynomial_generators = (0..count)
            .map(|index| generator(1, index))
            .collect::<Vec<_>>();
        polynomial_generators[0] = Vec::new();

        assert_eq!(
            IpaGeneratorBasis::new(
                1,
                polynomial_generators,
                (0..count).map(|index| generator(2, index)).collect(),
                vec![9],
            ),
            Err(IpaGeneratorBasisError::EmptyGenerator {
                label: "polynomial",
                index: 0
            })
        );
    }

    #[test]
    fn generator_basis_rejects_all_zero_generator() {
        let count = expected_ipa_generator_count(1).unwrap();
        let mut evaluation_generators = (0..count)
            .map(|index| generator(2, index))
            .collect::<Vec<_>>();
        evaluation_generators[1] = vec![0, 0, 0];

        assert_eq!(
            IpaGeneratorBasis::new(
                1,
                (0..count).map(|index| generator(1, index)).collect(),
                evaluation_generators,
                vec![9],
            ),
            Err(IpaGeneratorBasisError::AllZeroGenerator {
                label: "evaluation",
                index: 1
            })
        );
    }

    #[test]
    fn generator_basis_rejects_duplicate_generator() {
        let count = expected_ipa_generator_count(1).unwrap();
        let polynomial_generators = (0..count)
            .map(|index| generator(1, index))
            .collect::<Vec<_>>();
        let mut evaluation_generators = (0..count)
            .map(|index| generator(2, index))
            .collect::<Vec<_>>();
        evaluation_generators[0] = polynomial_generators[1].clone();

        assert_eq!(
            IpaGeneratorBasis::new(1, polynomial_generators, evaluation_generators, vec![9],),
            Err(IpaGeneratorBasisError::DuplicateGenerator {
                label: "evaluation",
                index: 0
            })
        );
    }

    #[test]
    fn generator_basis_binding_is_deterministic() {
        let basis = basis(2);

        assert_eq!(bound_challenge(&basis), bound_challenge(&basis));
    }

    #[test]
    fn generator_basis_binding_changes_when_generator_changes() {
        let a = basis(2);
        let mut b = basis(2);
        b.polynomial_generators[0] = vec![7, 7, 7];

        assert_ne!(bound_challenge(&a), bound_challenge(&b));
    }
}
