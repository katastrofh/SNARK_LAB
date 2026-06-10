//! Zerocheck reduced to one weighted sumcheck instance.
use field::Fp;
use multilinear::{eq_evaluations, Multilinear};
use sumcheck::{prove, verify, Proof, VerifyError};

#[derive(Clone, Debug)]
pub struct ZeroCheckProof {
    pub mixing_point: Vec<Fp>,
    pub sumcheck: Proof,
}

pub fn weighted_polynomial(
    constraints: &Multilinear,
    mixing_point: &[Fp],
) -> Result<Multilinear, &'static str> {
    if mixing_point.len() != constraints.variables() {
        return Err("mixing point dimension mismatch");
    }
    let weighted = constraints
        .evaluations()
        .iter()
        .copied()
        .zip(eq_evaluations(mixing_point))
        .map(|(f, eq)| f * eq)
        .collect();
    Multilinear::new(weighted)
}

pub fn prove_zero(
    constraints: &Multilinear,
    mixing_point: Vec<Fp>,
) -> Result<ZeroCheckProof, &'static str> {
    let weighted = weighted_polynomial(constraints, &mixing_point)?;
    Ok(ZeroCheckProof {
        mixing_point,
        sumcheck: prove(&weighted, Fp::ZERO),
    })
}

pub fn verify_zero(constraints: &Multilinear, proof: &ZeroCheckProof) -> Result<(), VerifyError> {
    let weighted = weighted_polynomial(constraints, &proof.mixing_point)
        .map_err(|_| VerifyError::WrongRoundCount)?;
    verify(&weighted, &proof.sumcheck)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn zero_table_verifies() {
        let p = Multilinear::new(vec![Fp::ZERO; 8]).unwrap();
        let proof = prove_zero(&p, vec![2.into(), 3.into(), 5.into()]).unwrap();
        assert_eq!(verify_zero(&p, &proof), Ok(()));
    }
    #[test]
    fn nonzero_table_is_rejected() {
        let p = Multilinear::new(vec![0.into(), 0.into(), 9.into(), 0.into()]).unwrap();
        let proof = prove_zero(&p, vec![2.into(), 3.into()]).unwrap();
        assert!(verify_zero(&p, &proof).is_err());
    }
}
