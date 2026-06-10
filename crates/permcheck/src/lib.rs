//! Product and rational fingerprints for permutation checks.
use field::Fp;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PermCheckError {
    LengthMismatch,
    Pole,
}

pub fn product_fingerprint(values: &[Fp], beta: Fp) -> Fp {
    values.iter().map(|&value| beta + value).product()
}

/// A logarithmic-derivative fingerprint: Σ 1/(β + aᵢ).
pub fn rational_fingerprint(values: &[Fp], beta: Fp) -> Result<Fp, PermCheckError> {
    values.iter().try_fold(Fp::ZERO, |sum, &value| {
        (beta + value)
            .inverse()
            .map(|inverse| sum + inverse)
            .ok_or(PermCheckError::Pole)
    })
}

pub fn product_check(left: &[Fp], right: &[Fp], beta: Fp) -> Result<bool, PermCheckError> {
    if left.len() != right.len() {
        return Err(PermCheckError::LengthMismatch);
    }
    Ok(product_fingerprint(left, beta) == product_fingerprint(right, beta))
}

pub fn rational_check(left: &[Fp], right: &[Fp], beta: Fp) -> Result<bool, PermCheckError> {
    if left.len() != right.len() {
        return Err(PermCheckError::LengthMismatch);
    }
    Ok(rational_fingerprint(left, beta)? == rational_fingerprint(right, beta)?)
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

/// A simple, explicit cost model used by the UI and benchmark CLI.
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
    #[test]
    fn permutations_match() {
        let a = [1.into(), 5.into(), 9.into(), 2.into()];
        let b = [9.into(), 2.into(), 1.into(), 5.into()];
        assert_eq!(product_check(&a, &b, 11.into()), Ok(true));
        assert_eq!(rational_check(&a, &b, 11.into()), Ok(true));
    }
    #[test]
    fn mutation_fails() {
        let a = [1.into(), 2.into(), 3.into()];
        let b = [1.into(), 2.into(), 4.into()];
        assert_eq!(product_check(&a, &b, 10.into()), Ok(false));
        assert_eq!(rational_check(&a, &b, 10.into()), Ok(false));
    }
    #[test]
    fn rational_model_is_streaming() {
        assert!(
            estimate_rational_stream(1 << 20, 32).bytes_read
                < estimate_product_tree(1 << 20, 32).bytes_read
        );
    }
}
