# Oracle Proof Serialization

This document defines canonical byte encoding for the transparent oracle-backed Sumcheck proof.

## Encoded object

The encoded object is:

    OracleProof<F, TransparentOracle<F>>

It contains:

    magic bytes
    field modulus bit size
    transparent commitment variables
    transparent commitment evaluations
    encoded inner Sumcheck proof
    transparent final opening value

## Why this exists

The previous codec serialized only the inner `sumcheck::Proof<F>`.

The oracle-backed verifier needs the complete object:

    commitment
    Sumcheck proof
    final opening

so this codec makes the transparent oracle-backed proof portable as bytes.

## Scope

This is still not a succinct proof format because `TransparentOracle` commitments contain the full evaluation table.

The purpose is to stabilize the byte-level interface before replacing the transparent backend with KZG, IPA, or FRI.

## Decoder rejection cases

The decoder rejects:

    wrong magic
    wrong field modulus size
    non-canonical field bytes
    truncated input
    trailing bytes
    malformed inner Sumcheck proof
