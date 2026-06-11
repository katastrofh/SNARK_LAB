# IPA Prover Commit Path

This document defines the prover-side commit path for the IPA backend work.

## What this adds

The oracle crate now exposes:

    IpaCurveProverKey
    IpaProverCommitment
    IpaProverCommitError
    commit_with_ipa_prover_key

## Commitment equation

The commit path computes:

    C = <a, G> + rH

where:

    a is the multilinear evaluation vector
    G is the polynomial generator basis
    r is the explicit blinding scalar
    H is the blinding generator

## Production boundary

This is a real curve commitment computation.

It is still not a full IPA opening proof system. It does not produce reduction rounds, and it does not verify hidden polynomial openings.

## Why this branch exists

Earlier branches defined:

    curve point types
    generator basis validation
    commitment equation

This branch connects those components into a clean prover-facing commit API.

## Security rule

Do not expose the polynomial vector or blinding scalar to public verification logic.

The public verifier must eventually check an IPA opening proof, not recompute a hidden-witness commitment.
