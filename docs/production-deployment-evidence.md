# Production Deployment Evidence

This document lists evidence required before SNARK_LAB can be described as production-secure deployment software.

## Current status

Current status:

    production-grade research prototype

Not yet claimed:

    audited
    production-secure
    mainnet-ready
    custody-safe
    consensus-critical safe

## Required evidence

### Cryptographic audit

Required:

- external reviewer or audit firm
- audit scope document
- commit hash under review
- findings report
- severity classification
- remediation commits
- final review status

### Production SRS ceremony

Required:

- ceremony design
- participant instructions
- transcript format
- randomness contribution method
- toxic-waste destruction statement
- final SRS artifact
- final SRS digest
- verification tool
- public ceremony transcript
- reproducible SRS validation command

### Side-channel review

Required:

- secret/public classification
- constant-time review
- dependency arithmetic review
- RNG review
- logging review
- CLI secret-handling review

### Fuzzing evidence

Required:

- campaign duration
- target names
- machine/environment
- crash count
- minimized regressions
- final campaign logs
- remediation notes

### Release reproducibility

Required:

- clean tag
- reproducible build command
- artifact checksums
- locked dependencies
- release notes
- public test vectors
- CI run links

## Deployment gate

A production deployment should not proceed unless all of the following are true:

- production gate passes
- GitHub Actions pass
- audit is complete
- side-channel review is complete
- SRS ceremony evidence exists
- fuzz campaign evidence exists
- release artifacts are checksummed
- deployment configuration is reviewed
- rollback procedure exists

## Statement policy

Until the above evidence exists, use:

    production-grade research prototype

Do not use:

    production-secure
    audited
    mainnet-ready
    custody-safe

## SRS ceremony manifest verification

Required command:

    python3 scripts/verify-srs-ceremony-manifest.py <manifest.json> --strict-production

The production SRS manifest must not contain placeholder digests.

## Evidence pack generation

Generate release/deployment evidence with:

    scripts/collect-deployment-evidence.sh

The generated pack records commit hash, git status, toolchain versions, gate outputs, public vector checks, SRS manifest checks, dependency tree, and artifact digests.
