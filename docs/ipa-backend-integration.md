# IPA Backend Integration

This document defines the real typed IPA backend API.

## What this adds

The oracle crate now exposes:

    trim_ipa_integrated_keys
    commit_ipa_backend
    open_ipa_backend
    verify_ipa_backend

and typed key/opening/witness objects:

    IpaIntegratedProverKey
    IpaIntegratedVerifierKey
    IpaIntegratedCommitmentWitness
    IpaIntegratedOpening

## Production boundary

The old `IpaBackend` trait implementation remains shape-only because that trait does not carry curve generator material or explicit commitment blinding.

This branch does not fake that API.

The new typed backend is the real backend path. It requires explicit generator material and explicit blinding.

## Security rule

The blinding scalar is prover witness material.

Do not serialize `IpaIntegratedCommitmentWitness` as a public proof object.

Generator material must be derived or loaded with proper domain separation.
