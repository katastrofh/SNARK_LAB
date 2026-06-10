//! Production-sized field choices for the Rust protocol core.
//!
//! Browser-only educational examples use F_97 in TypeScript. Rust protocols use
//! Arkworks prime fields and default to the BLS12-381 scalar field.

pub use ark_bls12_381::Fr as BlsScalar;
pub use ark_ff::{Field, PrimeField};

/// The default field used by examples, tests, the CLI, and benchmarks.
pub type DefaultField = BlsScalar;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_field_is_255_bits() {
        assert_eq!(DefaultField::MODULUS_BIT_SIZE, 255);
    }
}
