# IPA Generator Folding

This document defines generator folding for one IPA reduction round.

## Folding equations

The polynomial generators are folded as:

    G' = x^{-1} G_L + x G_R

The evaluation generators are folded as:

    H' = x H_L + x^{-1} H_R

These equations are paired with the vector folds:

    a' = x a_L + x^{-1} a_R
    b' = x^{-1} b_L + x b_R

## What this adds

The oracle crate now exposes:

    fold_ipa_polynomial_generators
    fold_ipa_evaluation_generators
    fold_ipa_generator_basis
    IpaGeneratorFoldingError

## Production boundary

This is real IPA folding algebra.

It is still not a full verifier. The next steps are to combine round commitments, vector folding, generator folding, and transcript-derived challenges into a full prover opening loop.
