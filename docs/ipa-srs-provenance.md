# IPA SRS Provenance

This branch adds a fail-closed provenance layer for IPA generator bases.

## What this adds

The oracle crate exposes:

    canonical_ipa_srs_digest
    validate_ipa_srs_provenance
    IpaSrsProvenance
    IpaSrsSource
    IpaVerifiedSrs

## Production rule

Production IPA SRS material must be either:

    externally supplied trusted setup material with source metadata and artifact digest

or:

    hash-to-curve-derived material with domain-separation metadata and derivation transcript digest

The following source is rejected by production validation:

    KnownDiscreteLogTestFixture

## What the digest binds

The canonical digest binds:

    digest domain version
    variable count
    polynomial generators
    evaluation generators
    blinding generator

All curve points are already canonical compressed points validated by the typed curve-point layer.

## Boundary

This branch does not implement hash-to-curve generation.

It validates externally supplied or externally derived SRS material and rejects known-discrete-log test fixtures from production provenance.
