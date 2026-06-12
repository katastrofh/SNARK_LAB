# IPA Proof Codec Integration

This document defines canonical serialization for the public integrated IPA opening object.

## Serialized object

The codec serializes:

    IpaIntegratedOpening {
        claimed_value,
        proof
    }

The claimed value is encoded through the canonical IPA proof payload and reconstructed during decoding.

## Not serialized

The codec does not serialize:

    IpaIntegratedCommitmentWitness
    commitment blinding scalar
    prover-only witness material

## What this adds

The oracle crate now exposes:

    encode_ipa_integrated_opening
    decode_ipa_integrated_opening
    IpaBackendOpeningCodecError

## Production boundary

This is public proof/opening serialization only.

It is not key serialization and it does not serialize prover secrets.

## Security rule

Never serialize `IpaIntegratedCommitmentWitness` as a public proof artifact.
