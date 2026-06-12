# IPA Prover Opening Loop

This document defines the prover-side IPA opening loop.

## What this adds

The oracle crate now exposes:

    prove_ipa_opening
    IpaProverOpeningOutput
    IpaProverOpeningError

The loop performs:

    opening statement binding
    L/R round commitment computation
    Fiat-Shamir challenge derivation
    polynomial vector folding
    evaluation vector folding
    generator basis folding
    final scalar extraction
    proof-shape construction

## Production boundary

This is a real prover-side construction path.

It is not a verifier and it never returns an acceptance decision.

The next production step is to implement the verifier-side recursive relation:

    P' = P + x^2 L + x^{-2} R

and then check the final scalar and commitment relation.

## Security rule

Do not expose this as a complete PCS backend until the verifier loop exists and all malformed-proof rejection tests pass.
