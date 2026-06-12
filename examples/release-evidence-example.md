# Release Evidence Example

SNARK_LAB records release evidence so reviewers can inspect what was built, tagged, and published.

## What to inspect

Look for:

- release notes
- release-candidate evidence
- GitHub Release publication evidence
- checksums
- tag information
- asset manifests

## Suggested files

    release/v0.2.0-rc.2.md
    release/GITHUB_RELEASE_PAGE_v0.2.0-rc.2.md
    release/publication/v0.2.0-rc.2/
    release-candidates/LATEST.md
    release-candidates/LATEST.json

## Suggested commands

    scripts/check-production-ready.sh

## Things to verify manually

- The release tag points to the expected commit.
- Assets match recorded checksums.
- Release evidence avoids stronger security claims.
