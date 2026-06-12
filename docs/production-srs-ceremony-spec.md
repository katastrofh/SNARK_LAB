# Production SRS Ceremony Specification

This document defines the production public-parameter ceremony specification for SNARK_LAB.

## Current status

This is a specification and verification framework.

It is not a completed production ceremony.

## Scheme

The target scheme is:

    ipa-multilinear-pcs

The target curve is:

    BLS12-381

The target scalar field is:

    Fr

## Trusted setup model

For the IPA commitment path, the preferred production parameter model is transparent generator derivation.

The ceremony should not require hidden toxic waste.

Instead, the production process should publish:

- generator derivation domain separator
- public randomness beacon or transcript root
- final SRS file
- final SRS digest
- manifest JSON
- validation command
- public transcript digest

## Required manifest

A production manifest must include:

- manifest version
- scheme
- curve
- field
- max variables
- generator derivation method
- public beacon
- final SRS file path
- final SRS SHA-256 digest
- verification command
- transcript SHA-256 digest
- participants or contributors
- production readiness status
- audit requirement status

## Manifest verifier

The manifest verifier is:

    scripts/verify-srs-ceremony-manifest.py

Example check:

    scripts/check-srs-ceremony-spec.sh

Strict production check:

    python3 scripts/verify-srs-ceremony-manifest.py <manifest.json> --strict-production

## Production ceremony steps

1. Freeze the protocol commit.
2. Freeze the SRS format version.
3. Choose the max variable count.
4. Choose the public randomness source.
5. Derive generator material using the published domain separator.
6. Serialize the final SRS file.
7. Compute SHA-256 digest of the SRS file.
8. Publish the manifest.
9. Publish the transcript digest.
10. Run the manifest verifier.
11. Run the SRS validation CLI.
12. Attach the manifest and SRS digest to the release.
13. Include the ceremony status in release notes.

## Required release evidence

A production release must include:

- SRS manifest
- final SRS digest
- transcript digest
- verifier command output
- production gate output
- release artifact SHA-256 digests
- audit status
- side-channel status

## Rejection criteria

Reject a production SRS manifest if:

- it uses placeholder digests
- it lacks a public randomness source
- it lacks a final SRS digest
- it lacks a verification command
- it claims production readiness without audit status
- it expects hidden toxic waste for the transparent IPA path
- the final SRS digest does not match the referenced file

## Security boundary

This specification is part of production readiness.

It does not by itself prove that a production ceremony has happened.

A production ceremony is complete only when real artifacts, digests, transcript evidence, and verification outputs are published.
