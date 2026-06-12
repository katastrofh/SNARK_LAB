# SNARK_LAB SRS Ceremony

This directory defines the production public-parameter ceremony process for SNARK_LAB.

## Current status

The checked-in manifest is an example.

It is not a production ceremony artifact.

## IPA PCS parameter model

SNARK_LAB's IPA polynomial commitment path uses public generator material.

The preferred production route is transparent parameter generation:

- domain-separated generator derivation
- public randomness beacon or ceremony transcript root
- public manifest
- public digest
- reproducible verifier command

Unlike toxic-waste trusted setup systems, this IPA parameter flow should not require a hidden trapdoor.

## Required production artifacts

A production ceremony must publish:

- final SRS file
- final SRS SHA-256 digest
- manifest JSON
- transcript digest
- public randomness source
- generator derivation method
- max supported variables
- verification command
- participant/contributor log, if any
- audit/review status

## Example manifest

The example manifest is:

    ceremony/production-srs-manifest.example.json

Validate it with:

    scripts/check-srs-ceremony-spec.sh

## Production rule

Do not claim a production SRS ceremony has happened until a real manifest and SRS artifact are published and externally reviewable.
