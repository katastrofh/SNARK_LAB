# Reviewer Onboarding Guide

This guide is for reviewers who want to inspect SNARK_LAB without guessing where to start.

## First five minutes

Start with:

1. `README.md`
2. `docs/project-positioning.md`
3. `ROADMAP.md`
4. `docs/final-repo-health-report.md`
5. `release/GITHUB_RELEASE_PAGE_v0.2.0-rc.2.md`

These files explain the project status, release-candidate state, security boundary, and evidence stack.

## Recommended local checks

Run:

    scripts/check-production-ready.sh

This is the main repository evidence gate.

It checks formatting, linting, tests, fuzz target compilation, public vectors, release evidence, SRS policy, deployment templates, visualizer build, and documentation evidence.

## Protocol areas to review

Suggested review order:

1. Sumcheck
2. Zerocheck
3. PermCheck
4. multilinear evaluation utilities
5. IPA commitment path
6. IPA proof serialization
7. SRS provenance and loader boundaries
8. fuzz targets and regression fixtures
9. public test vectors
10. visualizer flow

## Evidence areas to inspect

Important evidence directories and files:

- `release-candidates/`
- `release/publication/`
- `fuzz/smoke-evidence/`
- `fuzz/regressions/`
- `test-vectors/`
- `deployment/evidence/`
- `audits/packet/`
- `srs/PRODUCTION_SRS_POLICY.md`
- `docs/final-repo-health-report.md`

## Release artifacts

The current main-branch release candidate is:

    v0.2.0-rc.2

The GitHub Release evidence is recorded in:

    release/publication/v0.2.0-rc.2/

## What to verify manually

Reviewers should manually check:

- the release tag points to the expected commit
- release assets match the recorded checksums
- negative fixtures actually reject tampering
- fuzz regression fixtures are kept as regression tests
- SRS examples are not presented as production ceremony outputs
- visualizer text does not overstate the implementation status
- documentation keeps the research-prototype boundary clear

## Security boundary

SNARK_LAB is a protocol lab and research prototype.

It is appropriate for study, review, demonstrations, and reproducibility checks.

It is not a deployment-grade cryptographic library.

Before any stronger deployment claim, the project needs outside cryptographic review, side-channel review, longer fuzzing, dependency review, and SRS ceremony artifact review.
