//! Dense multilinear extensions in little-endian Boolean-cube order.
use field::Fp;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Multilinear {
    evaluations: Vec<Fp>,
    variables: usize,
}

impl Multilinear {
    pub fn new(evaluations: Vec<Fp>) -> Result<Self, &'static str> {
        if evaluations.is_empty() || !evaluations.len().is_power_of_two() {
            return Err("evaluation count must be a non-zero power of two");
        }
        Ok(Self {
            variables: evaluations.len().ilog2() as usize,
            evaluations,
        })
    }
    pub fn evaluations(&self) -> &[Fp] {
        &self.evaluations
    }
    pub fn variables(&self) -> usize {
        self.variables
    }
    pub fn sum_hypercube(&self) -> Fp {
        self.evaluations.iter().copied().sum()
    }
    pub fn evaluate(&self, point: &[Fp]) -> Result<Fp, &'static str> {
        if point.len() != self.variables {
            return Err("point dimension mismatch");
        }
        let mut layer = self.evaluations.clone();
        for &r in point {
            layer = layer
                .chunks_exact(2)
                .map(|pair| pair[0] * (Fp::ONE - r) + pair[1] * r)
                .collect();
        }
        Ok(layer[0])
    }
    pub fn fold_first(&self, challenge: Fp) -> Self {
        Self::new(
            self.evaluations
                .chunks_exact(2)
                .map(|p| p[0] * (Fp::ONE - challenge) + p[1] * challenge)
                .collect(),
        )
        .unwrap()
    }
}

/// eq(r, x) = Π_i (r_i x_i + (1-r_i)(1-x_i)).
pub fn eq_evaluations(point: &[Fp]) -> Vec<Fp> {
    let mut values = vec![Fp::ONE];
    for &r in point {
        let mut next = Vec::with_capacity(values.len() * 2);
        for value in values {
            next.push(value * (Fp::ONE - r));
            next.push(value * r);
        }
        values = next;
    }
    values
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn interpolates() {
        let p = Multilinear::new(vec![0.into(), 2.into(), 4.into(), 8.into()]).unwrap();
        assert_eq!(p.evaluate(&[1.into(), 1.into()]).unwrap(), 8.into());
    }
    #[test]
    fn eq_sums_to_one() {
        assert_eq!(
            eq_evaluations(&[3.into(), 7.into(), 11.into()])
                .into_iter()
                .sum::<Fp>(),
            Fp::ONE
        );
    }
}
