# IPA Blinding Opening Extension

This document defines the algebraic extension needed to open blinded IPA commitments without fake verifier acceptance.

## Commitment

A blinded commitment has the form:

    C = <a, G> + rB

where `r` is the commitment blinding scalar and `B` is the commitment blinding generator.

## Extension

The blinded relation is converted into a larger ordinary IPA relation:

    a_ext = (a, r, 0, ..., 0)
    G_ext = (G, B, padding generators)
    b_ext = (eq(z, ·), 0, ..., 0)

Then:

    C = <a_ext, G_ext>
    v = <a_ext, b_ext>

## What this adds

The oracle crate now exposes:

    IpaBlindedOpeningExtension
    IpaBlindingExtensionError
    extend_ipa_opening_for_blinding

## Production boundary

This is a real algebraic extension for hiding support.

It does not yet connect to `prove_ipa_opening` or `verify_ipa_opening`. The next branch must integrate the extended relation into the prover and verifier paths.

## Security rule

Padding generators and the carried-forward extended blinding generator must be independently derived or supplied with domain separation.

Do not reuse generator labels across `G`, `H`, padding generators, and the final basis blinding generator.
