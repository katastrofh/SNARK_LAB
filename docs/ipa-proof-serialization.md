# IPA Proof Serialization

This document defines canonical byte serialization for the typed IPA opening proof object.

## What this adds

The oracle crate now exposes:

    encode_ipa_opening_proof
    decode_ipa_opening_proof
    IpaProofCodecError

## Encoded object

The encoded object is:

    IpaOpeningProof<F>

The byte format binds:

    magic bytes
    field modulus bit size
    variable count
    claimed opening value
    reduction rounds
    final folded polynomial scalar
    final folded evaluation-basis scalar
    final commitment bytes

## Production boundary

This is serialization only.

It does not verify IPA group equations.

## Decoder rejection cases

The decoder rejects:

    wrong magic
    wrong field modulus size
    non-canonical field encodings
    truncated input
    trailing bytes
    malformed transcript rounds
    invalid proof shape

## Security rule

Do not treat successful decoding as cryptographic verification.

A decoded proof is only structurally well-formed. A real verifier must check the IPA equations.
