use ark_ff::PrimeField;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DenseRoundPolynomial<F: PrimeField> {
    pub coefficients: Vec<F>,
}

impl<F: PrimeField> DenseRoundPolynomial<F> {
    pub fn new(coefficients: Vec<F>) -> Self {
        Self { coefficients }
    }

    pub fn constant(value: F) -> Self {
        Self {
            coefficients: vec![value],
        }
    }

    pub fn linear(evaluation_at_zero: F, evaluation_at_one: F) -> Self {
        Self {
            coefficients: vec![evaluation_at_zero, evaluation_at_one - evaluation_at_zero],
        }
    }

    pub fn degree(&self) -> usize {
        self.coefficients
            .iter()
            .rposition(|coefficient| !coefficient.is_zero())
            .unwrap_or(0)
    }

    pub fn evaluate(&self, point: F) -> F {
        self.coefficients
            .iter()
            .rev()
            .fold(F::ZERO, |accumulator, coefficient| {
                accumulator * point + coefficient
            })
    }

    pub fn boolean_sum(&self) -> F {
        self.evaluate(F::ZERO) + self.evaluate(F::ONE)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ark_bls12_381::Fr;

    #[test]
    fn evaluates_dense_round_polynomial() {
        let polynomial = DenseRoundPolynomial::new(vec![Fr::from(3), Fr::from(2), Fr::from(5)]);

        assert_eq!(polynomial.degree(), 2);
        assert_eq!(polynomial.evaluate(Fr::from(0)), Fr::from(3));
        assert_eq!(polynomial.evaluate(Fr::from(1)), Fr::from(10));
        assert_eq!(polynomial.boolean_sum(), Fr::from(13));
    }

    #[test]
    fn linear_constructor_matches_endpoints() {
        let polynomial = DenseRoundPolynomial::linear(Fr::from(11), Fr::from(29));

        assert_eq!(polynomial.degree(), 1);
        assert_eq!(polynomial.evaluate(Fr::from(0)), Fr::from(11));
        assert_eq!(polynomial.evaluate(Fr::from(1)), Fr::from(29));
    }
}
