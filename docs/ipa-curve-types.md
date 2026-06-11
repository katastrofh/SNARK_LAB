# IPA Curve Types

This document introduces curve-aware IPA generator types.

## What this adds

The oracle crate now exposes:

    IpaCurvePoint
    IpaCurveGeneratorBasis
    IpaCurvePointError
    bind_ipa_curve_generator_basis

## Why this matters

Previous IPA layers used byte vectors as placeholders.

This branch introduces typed curve points and canonical compressed serialization through arkworks.

## Validation

The curve-point layer rejects:

    invalid compressed encodings
    identity points
    wrong generator counts
    duplicate generators

## Transcript binding

The typed generator basis is converted into canonical compressed bytes and bound through the existing IPA generator-basis transcript schedule.

## Production boundary

This is still not a full IPA verifier.

It gives the backend concrete curve-point types and serialization. A later branch must implement the actual commitment equation and IPA reduction checks.
