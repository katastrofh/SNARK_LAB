# IPA Evaluation Basis

This document defines the evaluation-basis vector used by IPA openings.

## Equation

For a multilinear polynomial represented by its Boolean-hypercube table `a`,
an opening claim at point `z` is:

    f(z) = <a, eq(z, ·)>

where `eq(z, ·)` is the equality-polynomial evaluation vector over the Boolean cube.

## What this adds

The oracle crate now exposes:

    IpaEvaluationBasis
    IpaEvaluationBasisError
    compute_ipa_evaluation_basis
    evaluate_with_ipa_evaluation_basis
    bind_ipa_evaluation_basis

## Ordering

The basis is ordered to match `Multilinear::evaluations()` and `Multilinear::evaluate()`.

The lower-level equality-vector helper expands coordinates in the opposite bit-significance order, so the IPA evaluation-basis constructor reverses the point before expansion.

## Production boundary

This does not yet produce an IPA opening proof.

It provides the right-hand vector for the inner-product opening relation.

## Security rule

Do not treat evaluation-basis computation as proof verification.

The verifier must later check that the committed polynomial vector opens to the claimed inner product without seeing the full vector.
