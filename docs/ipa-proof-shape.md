# IPA Proof Shape

This document defines the typed IPA opening proof object.

## What this adds

The oracle crate now exposes:

    IpaOpeningProof
    IpaProofShapeError
    validate_ipa_opening_proof_shape

## Proof shape

The proof contains:

    variable count
    claimed opening value
    IPA reduction rounds
    final folded polynomial scalar
    final folded evaluation-basis scalar
    final commitment bytes

## Production boundary

This is still not cryptographic verification.

The goal is to fix the proof object before implementing group arithmetic.

## Security rule

Do not accept an IPA proof because its shape is valid.

Shape validation only checks structural well-formedness:

    correct round count
    non-empty final commitment

A real verifier must additionally check the IPA group equations.
