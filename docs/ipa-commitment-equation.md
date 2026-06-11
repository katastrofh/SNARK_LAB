# IPA Commitment Equation

This document defines the first actual curve-level IPA commitment equation.

## Equation

The commitment equation is:

    C = <a, G> + rH

where:

    a is the multilinear evaluation vector
    G is the polynomial-generator basis
    r is the blinding scalar
    H is the blinding generator

## What this adds

The oracle crate now exposes:

    IpaCurveCommitment
    IpaCommitmentEquationError
    commit_ipa_polynomial
    check_ipa_commitment_equation
    validate_ipa_commitment_inputs

## Production boundary

This is not yet full IPA opening verification.

It computes and checks the commitment relation when the polynomial and blinding scalar are known.

A real verifier will not know the polynomial vector or blinding scalar. A later IPA opening verifier must check the reduction proof instead.

## Validation

The implementation rejects:

    variable-count mismatch
    invalid curve generator basis
    generator-count mismatch

## Security rule

Do not treat `check_ipa_commitment_equation` as a public verifier for hidden witnesses.

It is a witness-side consistency check and a building block for the prover path.
