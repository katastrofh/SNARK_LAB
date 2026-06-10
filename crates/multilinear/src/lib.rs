//! Dense multilinear extensions in little-endian Boolean-cube order.

use ark_ff::Field;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Multilinear<F: Field> {
    evaluations: Vec<F>,
    variables: usize,
}

impl<F: Field> Multilinear<F> {
    pub fn new(evaluations: Vec<F>) -> Result<Self, &'static str> {
        if evaluations.is_empty() || !evaluations.len().is_power_of_two() {
            return Err("evaluation count must be a non-zero power of two");
        }
        Ok(Self {
            variables: evaluations.len().ilog2() as usize,
            evaluations,
        })
    }

    pub fn evaluations(&self) -> &[F] {
        &self.evaluations
    }

    pub fn variables(&self) -> usize {
        self.variables
    }

    pub fn sum_hypercube(&self) -> F {
        self.evaluations.iter().copied().sum()
    }

    pub fn evaluate(&self, point: &[F]) -> Result<F, &'static str> {
        if point.len() != self.variables {
            return Err("point dimension mismatch");
        }
        let mut layer = self.evaluations.clone();
        for &challenge in point {
            layer = layer
                .chunks_exact(2)
                .map(|pair| pair[0] + challenge * (pair[1] - pair[0]))
                .collect();
        }
        Ok(layer[0])
    }

    pub fn fold_first(&self, challenge: F) -> Self {
        Self::new(
            self.evaluations
                .chunks_exact(2)
                .map(|pair| pair[0] + challenge * (pair[1] - pair[0]))
                .collect(),
        )
        .expect("folding a valid multilinear table remains non-empty")
    }
}

/// `eq(r, x) = Π_i (r_i x_i + (1-r_i)(1-x_i))` on the Boolean cube.
pub fn eq_evaluations<F: Field>(point: &[F]) -> Vec<F> {
    let mut values = vec![F::ONE];
    for &coordinate in point {
        let mut next = Vec::with_capacity(values.len() * 2);
        for value in values {
            next.push(value * (F::ONE - coordinate));
            next.push(value * coordinate);
        }
        values = next;
    }
    values
}

#[cfg(test)]
mod tests {
    use super::*;
    use ark_bls12_381::Fr;

    #[test]
    fn interpolates() {
        let polynomial =
            Multilinear::new(vec![0_u64, 2, 4, 8].into_iter().map(Fr::from).collect()).unwrap();
        assert_eq!(
            polynomial.evaluate(&[Fr::ONE, Fr::ONE]).unwrap(),
            Fr::from(8)
        );
    }

    #[test]
    fn equality_polynomial_sums_to_one() {
        assert_eq!(
            eq_evaluations(&[Fr::from(3), Fr::from(7), Fr::from(11)])
                .into_iter()
                .sum::<Fr>(),
            Fr::ONE
        );
    }
}
