//! A transparent sumcheck transcript for multilinear evaluation tables.
use field::{Fp, MODULUS};
use multilinear::Multilinear;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Round {
    pub g_at_zero: Fp,
    pub g_at_one: Fp,
    pub challenge: Fp,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Proof {
    pub claimed_sum: Fp,
    pub rounds: Vec<Round>,
    pub final_evaluation: Fp,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VerifyError {
    WrongRoundCount,
    RoundDoesNotMatchClaim { round: usize },
    FinalEvaluationMismatch,
}

fn challenge(round: usize, claim: Fp, g0: Fp, g1: Fp) -> Fp {
    // A deterministic pedagogical transcript. Production systems use Fiat-Shamir.
    Fp::from(
        (claim.value() * 17 + g0.value() * 31 + g1.value() * 43 + round as u64 * 13 + 7) % MODULUS,
    )
}

pub fn prove(polynomial: &Multilinear, claimed_sum: Fp) -> Proof {
    let mut folded = polynomial.clone();
    let mut claim = claimed_sum;
    let mut rounds = Vec::with_capacity(polynomial.variables());
    for round in 0..polynomial.variables() {
        let g0: Fp = folded
            .evaluations()
            .chunks_exact(2)
            .map(|pair| pair[0])
            .sum();
        let g1: Fp = folded
            .evaluations()
            .chunks_exact(2)
            .map(|pair| pair[1])
            .sum();
        let r = challenge(round, claim, g0, g1);
        rounds.push(Round {
            g_at_zero: g0,
            g_at_one: g1,
            challenge: r,
        });
        claim = g0 * (Fp::ONE - r) + g1 * r;
        folded = folded.fold_first(r);
    }
    Proof {
        claimed_sum,
        rounds,
        final_evaluation: folded.evaluations()[0],
    }
}

pub fn verify(polynomial: &Multilinear, proof: &Proof) -> Result<(), VerifyError> {
    if proof.rounds.len() != polynomial.variables() {
        return Err(VerifyError::WrongRoundCount);
    }
    let mut claim = proof.claimed_sum;
    let mut point = Vec::with_capacity(proof.rounds.len());
    for (round, message) in proof.rounds.iter().enumerate() {
        if message.g_at_zero + message.g_at_one != claim
            || message.challenge != challenge(round, claim, message.g_at_zero, message.g_at_one)
        {
            return Err(VerifyError::RoundDoesNotMatchClaim { round });
        }
        claim = message.g_at_zero * (Fp::ONE - message.challenge)
            + message.g_at_one * message.challenge;
        point.push(message.challenge);
    }
    if claim != proof.final_evaluation
        || polynomial.evaluate(&point).unwrap() != proof.final_evaluation
    {
        return Err(VerifyError::FinalEvaluationMismatch);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn honest_proof_verifies() {
        let p = Multilinear::new((0..8).map(Fp::from).collect()).unwrap();
        let proof = prove(&p, p.sum_hypercube());
        assert_eq!(verify(&p, &proof), Ok(()));
    }
    #[test]
    fn false_claim_fails() {
        let p = Multilinear::new((0..4).map(Fp::from).collect()).unwrap();
        let proof = prove(&p, 42.into());
        assert!(verify(&p, &proof).is_err());
    }
}
