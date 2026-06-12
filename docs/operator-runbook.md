# Operator Runbook

This runbook describes how an operator should handle SNARK_LAB release-candidate artifacts.

## 1. Identify release

Check tag:

    git tag -n99 v0.2.0-rc.1

Check remote tag:

    git ls-remote --tags origin | grep v0.2.0-rc.1

## 2. Verify clean checkout

Run:

    git switch main
    git pull origin main
    git status

Expected:

    nothing to commit, working tree clean

## 3. Run production gate

Run:

    scripts/check-production-ready.sh

Expected:

    production readiness checks passed

## 4. Build release artifacts

Run:

    scripts/build-github-release-artifacts.sh v0.2.0-rc.1

## 5. Verify checksums

Run:

    cd dist/releases/v0.2.0-rc.1
    sha256sum -c SHA256SUMS

## 6. Inspect release evidence

Read:

    release-candidates/LATEST.md
    release-candidates/LATEST.json
    release/v0.2.0-rc.1.md
    release/GITHUB_RELEASE_DRAFT_v0.2.0-rc.1.md

## 7. Check production blockers

Confirm whether these are complete:

- external audit
- side-channel review
- production SRS artifact
- production SRS digest
- production ceremony transcript
- long fuzz campaign evidence

If any are incomplete, the release is not production-secure.

## 8. Allowed publication

For v0.2.0-rc.1, publish as:

    pre-release
    release-candidate
    not production-secure

## 9. Forbidden publication

Do not publish as:

    audited
    production-secure
    ready for mainnet deployment
    safe for custody use
    consensus-critical safe

## 10. Rollback

If an artifact or release note is wrong:

1. remove or correct the GitHub release
2. do not move the existing tag unless absolutely necessary
3. create a new rc tag if required
4. document the correction in CHANGELOG.md
