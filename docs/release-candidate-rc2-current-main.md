# Release Candidate rc2 from Current Main

This document records the v0.2.0-rc.2 release candidate process.

## Purpose

v0.2.0-rc.1 was published successfully, but it was tagged before later hardening and presentation branches.

v0.2.0-rc.2 should be tagged from current main so the release candidate includes:

- README star polish
- repository topic and badge polish
- visualizer screenshot assets
- all-target fuzz smoke evidence
- fuzz crash regression suite
- GitHub Release publication evidence
- production deployment guide
- production SRS placeholder policy

## Files

- `release/v0.2.0-rc.2.md`
- `release/GITHUB_RELEASE_PAGE_v0.2.0-rc.2.md`
- `scripts/check-rc2-current-main-release-candidate.sh`

## Boundary

This release candidate is for review, reproducibility checks, and protocol study.

It does not claim production security.
