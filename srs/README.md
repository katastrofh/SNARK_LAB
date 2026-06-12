# SRS Artifact Policy

This directory documents SNARK_LAB SRS artifact policy.

## Current status

SNARK_LAB does not commit production SRS artifacts into the repository.

The repository may contain:

- SRS policy documents
- SRS manifest examples
- SRS validation scripts
- ceremony templates

The repository must not contain:

- fake production SRS binaries
- placeholder production SRS binaries
- trusted-setup toxic-waste artifacts
- files that imply production SRS completion without evidence

## Production SRS evidence

Production SRS evidence belongs in release artifacts, not as an ambiguous checked-in placeholder.

A production release must provide:

- SRS artifact
- SHA-256 digest
- manifest
- ceremony or derivation transcript
- verifier output
- release evidence
