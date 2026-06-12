# Release Process

This document defines the release process for SNARK_LAB.

## Release boundary

SNARK_LAB releases are research-preview releases unless explicitly stated otherwise.

The repository is not audited and must not be used for production funds, custody, mainnet systems, or consensus-critical infrastructure.

## Pre-release checklist

Before tagging a release:

- Run the production gate locally
- Confirm GitHub Actions are green on main
- Confirm no untracked files
- Confirm Cargo.lock is committed
- Confirm web/visualizer/package-lock.json is committed
- Confirm README and SECURITY are current
- Confirm CHANGELOG has an entry for the release
- Confirm VERSIONING.md still matches the release status
- Confirm no known-discrete-log SRS fixture is described as production material
- Confirm all new public byte formats reject malformed input

## Local commands

Run:

    git switch main
    git pull origin main
    scripts/check-production-ready.sh
    scripts/audit-dependencies.sh
    cargo run --release -p snark-lab-cli -- ipa-demo
    cargo run --release -p snark-lab-benches -- 12 4 2
    git status

Expected final state:

    nothing to commit, working tree clean

## Tagging

Create an annotated tag:

    git tag -a v0.2.0 -m "SNARK_LAB v0.2.0 research preview"
    git push origin v0.2.0

The release workflow runs on tags matching:

    v*.*.*

## Release artifacts

The release workflow builds and uploads:

- snark-lab-cli
- snark-lab-benches
- README.md
- SECURITY.md
- RELEASE.md
- VERSIONING.md
- CHANGELOG.md
- SHA256SUMS.txt

## Manual GitHub release

After the tag workflow succeeds:

1. Open GitHub Releases.
2. Create a release from the tag.
3. Mark it as a pre-release.
4. Attach the artifact bundle from GitHub Actions if desired.
5. Include the security boundary text.

## Required release warning

Every release must include:

    This is an unaudited research-preview release.
    Do not use it for production funds, custody, mainnet systems, or consensus-critical infrastructure.
