# Deployment Evidence

This directory defines how SNARK_LAB deployment/release evidence is collected.

## Purpose

Deployment evidence records the exact repository state, toolchain versions, checks, vectors, SRS manifest validation, and artifact digests used for a release candidate.

## Current status

This directory provides the process and templates.

Generated evidence packs are not committed by default.

## Generate evidence

Run:

    scripts/collect-deployment-evidence.sh

To skip the full production gate during a quick dry run:

    scripts/collect-deployment-evidence.sh --skip-gate

Generated packs are written under:

    deployment/evidence/

## Production rule

A production release must have a committed or externally archived evidence pack containing:

- commit hash
- clean git status
- toolchain versions
- production gate output
- public test vector output
- SRS manifest verification output
- dependency tree
- artifact checksums
- release notes
- audit status
- side-channel status
