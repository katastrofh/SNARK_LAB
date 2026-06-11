# Canonical Proof Serialization

This document defines the first canonical byte encoding for Sumcheck proofs.

## Why this exists

Production verifiers must consume stable bytes, not Rust-only in-memory structs.

The first encoded object is:

    sumcheck::Proof<F>

It contains:

    magic bytes
    field modulus bit size
    number of rounds
    round polynomial endpoint evaluations
    final evaluation

## Encoding rules

Integers are little-endian `u64`.

Field elements are encoded as fixed-width little-endian canonical field bytes using the field modulus bit size.

The decoder rejects:

    wrong magic
    wrong field size
    truncated input
    trailing bytes

## Scope

This is not yet the full proof object for the oracle-backed verifier.

The next serialization target is:

    OracleProof<F, TransparentOracle<F>>

which includes:

    oracle commitment
    Sumcheck proof bytes
    final opening bytes
