# Production SRS Artifact Placeholder Policy

This document defines how SNARK_LAB handles production SRS artifacts and placeholders.

## Added

- srs/README.md
- srs/PRODUCTION_SRS_POLICY.md
- srs/production-srs-status.example.json
- scripts/check-production-srs-placeholder-policy.sh

## Policy

SNARK_LAB must not commit fake production SRS artifacts.

Example manifests are allowed.

Production SRS artifacts must be published with:

- artifact file
- SHA-256 digest
- manifest
- ceremony or derivation transcript
- verifier output
- release artifact checksums

## Forbidden

The repository rejects obvious placeholder files such as:

- production.srs
- production-srs.bin
- trusted_setup.ptau
- powers_of_tau.ptau

## Current status

Current status:

    no-production-srs-in-repo

Production SRS completed:

    false

## Boundary

This policy improves release integrity.

It does not itself create production SRS material.
