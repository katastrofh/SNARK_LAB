# IPA Backend Skeleton

This document defines the initial production-honest IPA backend boundary.

## What this adds

The `snark-lab-oracle` crate now exposes:

    IpaBackend
    IpaPublicParameters
    IpaProverKey
    IpaVerifierKey
    IpaCommitment
    IpaOpening
    IpaBackendError

The backend implements the `MultilinearPcs` trait shape.

## What this does not do

This is not yet a cryptographic IPA implementation.

The backend deliberately returns:

    BackendNotImplemented

for cryptographic operations instead of pretending to verify.

## Why this is production-honest

A fake verifier is worse than no verifier.

This skeleton gives the repository a stable implementation target while making it impossible to accidentally treat the IPA backend as secure before the actual protocol is implemented.

## Next implementation requirements

A real IPA backend must add:

    group element commitment type
    generator derivation or setup
    inner-product proof transcript
    prover reduction rounds
    verifier reduction checks
    canonical serialization
    binding and opening soundness documentation

## Security rule

Do not change `BackendNotImplemented` into success until the backend performs real cryptographic verification.
