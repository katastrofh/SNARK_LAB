# Polynomial Commitment Backend Traits

This document defines the production-facing PCS boundary for SNARK Lab.

## Why this exists

The current `MultilinearOracle` trait is protocol-side plumbing.

It is enough for Sumcheck to be commitment-shaped, but it is not enough for a real cryptographic backend because it does not expose:

    public parameters
    prover key
    verifier key
    backend setup
    backend trimming

The new `MultilinearPcs` trait adds that missing backend boundary.

## Required backend semantics

A real backend must implement:

    setup
    trim
    commit
    bind_commitment
    open
    verify

The verifier path must only depend on:

    verifier key
    commitment
    evaluation point
    opening proof

and must return the opened field value if verification succeeds.

## Intended backends

This trait is designed for future:

    KZG backend
    IPA backend
    FRI backend

## Current status

This branch adds the interface only.

It deliberately does not add a fake cryptographic backend.

The existing transparent oracle remains useful for protocol tests, but it is not a succinct commitment scheme.

## Production requirement

Any future backend must document:

    setup assumptions
    binding assumptions
    opening soundness
    supported domain
    trusted setup or transparent setup model
    serialization format
