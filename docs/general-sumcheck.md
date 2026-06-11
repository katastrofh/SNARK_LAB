# General Sumcheck API

This document describes the production-shaped Sumcheck split in snark-lab.

## Scope

The existing `sumcheck::prove` and `sumcheck::verify` functions remain the optimized multilinear-table path.

A vector of length `2^n` is treated as the Boolean-hypercube evaluation table of a multilinear extension.

The new general API adds bounded-degree round polynomials and a trait boundary for more general polynomial representations.

## Supported objects

| Object | Supported path |
|---|---|
| Vector of length `2^n` | Multilinear table path |
| Multilinear polynomial | Multilinear table path and general trait path |
| General bounded-degree multivariate polynomial | General trait path once it implements `SumcheckPolynomial` |
| Univariate polynomial | Degenerate one-variable Sumcheck |
| Arbitrary vector length | Not directly; pad or choose another domain |

## New types

    DenseRoundPolynomial<F>
    SumcheckPolynomial<F>
    GeneralProof<F>
    GeneralVerifyError

## New API

    prove_general(polynomial, claimed_sum, transcript)
    verify_general(polynomial, claimed_sum, proof, transcript)

## Degree checks

The verifier checks that every round polynomial has degree at most `max_individual_degree()`.

For multilinear tables, this bound is 1.

## Transcript separation

The general API uses a separate domain:

    snark-lab/sumcheck-general/v1

This prevents accidental proof reuse between the existing optimized multilinear-table verifier and the new general verifier.

## Production boundary

The general API is still transparent unless combined with the oracle/commitment layer.

A production SNARK verifier should eventually receive:

    commitment
    Sumcheck transcript
    final opening proof

not the full witness table.
