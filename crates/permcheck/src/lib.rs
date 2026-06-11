#![forbid(unsafe_code)]
//! Transcript-bound product and rational permutation fingerprints.

use ark_ff::PrimeField;
use snark_lab_transcript::ProofTranscript;

const DOMAIN: &[u8] = b"snark-lab/permcheck/v1";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    LengthMismatch,
    Pole,
}

#[derive(Clone, Copy, Debug)]
pub struct TaggedColumn<'a, F: PrimeField> {
    pub values: &'a [F],
    pub tags: &'a [F],
}

impl<F: PrimeField> TaggedColumn<'_, F> {
    fn validate(&self) -> Result<(), Error> {
        if self.values.len() != self.tags.len() {
            return Err(Error::LengthMismatch);
        }
        Ok(())
    }
}

fn bind_columns<F: PrimeField, T: ProofTranscript<F>>(
    left: TaggedColumn<'_, F>,
    right: TaggedColumn<'_, F>,
    transcript: &mut T,
) -> Result<(F, F), Error> {
    left.validate()?;
    right.validate()?;
    if left.values.len() != right.values.len() {
        return Err(Error::LengthMismatch);
    }

    transcript.append_domain_separator(DOMAIN);
    transcript.append_field_modulus();
    transcript.append_u64(b"column-length", left.values.len() as u64);
    for (side, column) in [(b"left".as_slice(), left), (b"right".as_slice(), right)] {
        transcript.append_bytes(b"column-side", side);
        for (&value, &tag) in column.values.iter().zip(column.tags) {
            transcript.append_field_element(b"column-value", &value);
            transcript.append_field_element(b"column-tag", &tag);
        }
    }
    let beta = transcript.challenge_scalar(b"beta");
    let gamma = transcript.challenge_scalar(b"gamma");
    Ok((beta, gamma))
}

fn compressed<F: PrimeField>(column: TaggedColumn<'_, F>, beta: F, gamma: F) -> Vec<F> {
    column
        .values
        .iter()
        .zip(column.tags)
        .map(|(&value, &tag)| value + beta * tag + gamma)
        .collect()
}

pub fn product_check<F: PrimeField, T: ProofTranscript<F>>(
    left: TaggedColumn<'_, F>,
    right: TaggedColumn<'_, F>,
    transcript: &mut T,
) -> Result<bool, Error> {
    let (beta, gamma) = bind_columns(left, right, transcript)?;
    Ok(compressed(left, beta, gamma).into_iter().product::<F>()
        == compressed(right, beta, gamma).into_iter().product::<F>())
}

pub fn rational_check<F: PrimeField, T: ProofTranscript<F>>(
    left: TaggedColumn<'_, F>,
    right: TaggedColumn<'_, F>,
    transcript: &mut T,
) -> Result<bool, Error> {
    let (beta, gamma) = bind_columns(left, right, transcript)?;
    let fingerprint = |column| {
        compressed(column, beta, gamma)
            .into_iter()
            .try_fold(F::ZERO, |sum, denominator| {
                denominator
                    .inverse()
                    .map(|inverse| sum + inverse)
                    .ok_or(Error::Pole)
            })
    };
    Ok(fingerprint(left)? == fingerprint(right)?)
}

pub fn product_fingerprint<F: PrimeField>(values: &[F], beta: F) -> F {
    values.iter().map(|&value| beta + value).product()
}

pub fn rational_fingerprint<F: PrimeField>(values: &[F], beta: F) -> Result<F, Error> {
    values.iter().try_fold(F::ZERO, |sum, &value| {
        (beta + value)
            .inverse()
            .map(|inverse| sum + inverse)
            .ok_or(Error::Pole)
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StreamEstimate {
    pub elements: usize,
    pub field_ops: usize,
    pub peak_field_elements: usize,
    pub passes: usize,
    pub bytes_read: usize,
    pub bytes_written: usize,
}

pub fn estimate_product_tree(elements: usize, field_bytes: usize) -> StreamEstimate {
    let levels = elements.max(1).next_power_of_two().ilog2() as usize;
    StreamEstimate {
        elements,
        field_ops: elements.saturating_sub(1),
        peak_field_elements: elements,
        passes: levels + 1,
        bytes_read: elements * field_bytes * (levels + 1),
        bytes_written: elements * field_bytes * levels,
    }
}

pub fn estimate_rational_stream(elements: usize, field_bytes: usize) -> StreamEstimate {
    StreamEstimate {
        elements,
        field_ops: elements * 2,
        peak_field_elements: 3,
        passes: 1,
        bytes_read: elements * field_bytes,
        bytes_written: 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ark_bls12_381::Fr;
    use ark_ff::AdditiveGroup;
    use snark_lab_transcript::MerlinTranscript;

    fn column(values: &[u64], tags: &[u64]) -> (Vec<Fr>, Vec<Fr>) {
        (
            values.iter().copied().map(Fr::from).collect(),
            tags.iter().copied().map(Fr::from).collect(),
        )
    }

    #[test]
    fn tagged_permutations_match() {
        let (left_values, left_tags) = column(&[1, 5, 9, 2], &[0, 1, 2, 3]);
        let (right_values, right_tags) = column(&[9, 2, 1, 5], &[2, 3, 0, 1]);
        let left = TaggedColumn {
            values: &left_values,
            tags: &left_tags,
        };
        let right = TaggedColumn {
            values: &right_values,
            tags: &right_tags,
        };
        let mut product_transcript = MerlinTranscript::new(b"permcheck-test");
        let mut rational_transcript = MerlinTranscript::new(b"permcheck-test");
        assert_eq!(
            product_check(left, right, &mut product_transcript),
            Ok(true)
        );
        assert_eq!(
            rational_check(left, right, &mut rational_transcript),
            Ok(true)
        );
    }

    #[test]
    fn mutation_fails() {
        let (left_values, left_tags) = column(&[1, 2, 3], &[0, 1, 2]);
        let (right_values, right_tags) = column(&[1, 2, 4], &[0, 1, 2]);
        let mut transcript = MerlinTranscript::new(b"permcheck-test");
        assert_eq!(
            product_check(
                TaggedColumn {
                    values: &left_values,
                    tags: &left_tags
                },
                TaggedColumn {
                    values: &right_values,
                    tags: &right_tags
                },
                &mut transcript
            ),
            Ok(false)
        );
    }

    #[test]
    fn denominator_poles_are_explicit() {
        assert_eq!(
            rational_fingerprint(&[Fr::ZERO], Fr::ZERO),
            Err(Error::Pole)
        );
    }
}
