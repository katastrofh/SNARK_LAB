# Production SRS Policy

## Policy

SNARK_LAB must not treat example SRS manifests or test fixtures as production SRS material.

Production SRS material must be either:

- externally supplied and verified
- generated through a documented transparent public-parameter derivation
- published as a release artifact with digest and manifest

## Forbidden repository state

The repository must not contain checked-in files that look like production SRS artifacts without evidence.

Forbidden examples:

- production.srs
- production-srs.bin
- trusted_setup.ptau
- powers_of_tau.ptau
- final-production-srs.bin

## Required evidence

Before production SRS can be accepted:

- artifact exists
- artifact digest exists
- manifest exists
- transcript exists
- verifier output exists
- release artifact checksum exists
- audit status is documented

## Current status

Current status:

    no-production-srs-in-repo

Production SRS completed:

    false
