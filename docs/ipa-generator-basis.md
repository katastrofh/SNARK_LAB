# IPA Generator Basis

This document defines the generator-basis boundary for a future IPA polynomial commitment backend.

## What this adds

The oracle crate now exposes:

    IpaGeneratorBasis
    IpaGeneratorBasisError
    expected_ipa_generator_count
    bind_ipa_generator_basis

## Basis shape

For a multilinear polynomial with `v` variables, the evaluation vector has:

    2^v

entries.

The basis therefore contains:

    2^v polynomial generators
    2^v evaluation generators
    1 blinding generator

## Validation

The basis rejects:

    wrong generator counts
    empty generator encodings
    all-zero generator encodings
    duplicate generator encodings
    variable counts that overflow machine indexing

## Transcript binding

The basis is bound into the Fiat-Shamir transcript before IPA opening proof challenges are derived.

This prevents a verifier from accepting a proof under a different generator basis than the one intended by the statement.

## Production boundary

This is not yet group arithmetic.

The byte vectors represent future canonical group-element encodings. A later branch must replace or wrap them with concrete curve-point types and subgroup checks.
