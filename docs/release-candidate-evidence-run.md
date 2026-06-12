# Release Candidate Evidence Run

This document describes how SNARK_LAB records release-candidate evidence.

## Added

- release-candidates/README.md
- scripts/summarize-latest-deployment-evidence.py
- scripts/check-release-candidate-evidence.sh

## Full evidence generation

Run:

    scripts/collect-deployment-evidence.sh

This writes a timestamped raw evidence pack under:

    deployment/evidence/

## Summary generation

Run:

    scripts/summarize-latest-deployment-evidence.py

This writes:

    release-candidates/LATEST.md
    release-candidates/LATEST.json

and timestamped summary files.

## Boundary

A release-candidate evidence run records what checks were executed.

It does not by itself establish production-secure status.

Production-secure status still requires:

- external audit
- side-channel review
- real production SRS ceremony artifacts
- production SRS digest
- release approval
