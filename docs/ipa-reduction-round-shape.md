# IPA Reduction Round Shape

This document defines the checked state for one IPA reduction round.

## Round state

Each round contains:

    L commitment
    R commitment
    challenge x
    inverse challenge x^{-1}
    input vector length
    output vector length

## Folding equations

The polynomial vector is folded as:

    a' = x a_L + x^{-1} a_R

The evaluation vector is folded as:

    b' = x^{-1} b_L + x b_R

## What this adds

The oracle crate now exposes:

    IpaReductionRound
    IpaReductionRoundError
    fold_ipa_polynomial_vector
    fold_ipa_evaluation_vector
    validate_ipa_vector_fold
    bind_ipa_reduction_round_context

## Production boundary

This branch does not claim full IPA proof verification.

It defines checked reduction-round state and folding algebra. Future branches must compute curve commitments L/R and verify the recursive commitment relation.
