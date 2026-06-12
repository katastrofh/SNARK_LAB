# GitHub Release Artifacts and Checksums

This document describes how to generate release artifacts and SHA-256 checksums for GitHub releases.

## Added

- scripts/build-github-release-artifacts.sh
- scripts/check-github-release-artifacts.sh
- release/GITHUB_RELEASE_DRAFT_v0.2.0-rc.1.md

## Generate artifacts

Run:

    scripts/build-github-release-artifacts.sh v0.2.0-rc.1

This writes artifacts under:

    dist/releases/v0.2.0-rc.1/

## Generated files

The artifact directory includes:

- source tarball
- source zip
- manifest
- tag information
- release notes
- release-candidate evidence files
- SRS manifest example
- security document
- changelog
- SHA256SUMS

## GitHub release process

1. Build artifacts.
2. Inspect SHA256SUMS.
3. Create GitHub release for the tag.
4. Paste release/GITHUB_RELEASE_DRAFT_v0.2.0-rc.1.md as release notes.
5. Attach generated artifacts.
6. Attach SHA256SUMS.
7. Mark release as pre-release.

## Boundary

Release artifacts and checksums improve reproducibility.

They do not replace:

- external audit
- side-channel review
- production SRS ceremony evidence
- production deployment approval
