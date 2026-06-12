# Deployment Evidence Pack

This document defines the SNARK_LAB deployment evidence pack.

## Purpose

The evidence pack records the exact checkout, toolchain, checks, vectors, SRS manifest validation, and artifact digests used for a release candidate or production deployment.

## Added

- deployment/README.md
- deployment/evidence/README.md
- deployment/templates/release-evidence-template.md
- deployment/templates/release-attestation-template.json
- scripts/collect-deployment-evidence.sh
- scripts/check-deployment-evidence-pack.sh

## Generate evidence

Run:

    scripts/collect-deployment-evidence.sh

For a fast process check without running the full production gate:

    scripts/collect-deployment-evidence.sh --skip-gate

## Evidence contents

A generated evidence pack includes:

- commit hash
- branch name
- git status
- environment versions
- cargo dependency tree
- public vector check output
- SRS manifest check output
- fuzz target compile output
- production gate output
- tracked artifact SHA-256 digests
- JSON attestation

## Production boundary

A generated evidence pack does not automatically mean the system is production-secure.

Production-secure deployment requires:

- external audit evidence
- side-channel review evidence
- production SRS ceremony evidence
- release artifact evidence
- clean production gate
- deployment approval

## Reviewer usage

A reviewer can inspect the evidence pack to verify what was actually run for a release candidate.

Do not accept screenshots or claims without logs, digests, and commit hashes.
