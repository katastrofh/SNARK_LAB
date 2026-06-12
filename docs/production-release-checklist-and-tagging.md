# Production Release Checklist and Tagging

This document describes SNARK_LAB release checklist and tagging policy.

## Added

- release/README.md
- release/PRODUCTION_RELEASE_CHECKLIST.md
- release/RELEASE_NOTES_TEMPLATE.md
- scripts/check-release-checklist.py
- scripts/check-release-checklist.sh
- scripts/prepare-release-tag.sh

## Release classes

SNARK_LAB distinguishes:

- research-preview
- release-candidate
- production-secure

## Current allowed status

Current status:

    release-candidate capable

Not yet claimed:

    production-secure

## Release checklist validation

Run:

    scripts/check-release-checklist.sh

This checks that the repository has release notes, release checklist, release-candidate evidence, deployment evidence process, audit packet, SRS manifest specification, and required scripts.

## Tag dry run

Run:

    scripts/prepare-release-tag.sh v0.2.0-rc.1

## Create annotated tag

Run:

    scripts/prepare-release-tag.sh v0.2.0-rc.1 --create

Then push with:

    git push origin v0.2.0-rc.1

## Production-secure requirement

Do not create a production-secure release until:

- external audit is complete
- critical/high findings are resolved
- side-channel review is complete
- production SRS ceremony evidence is published
- long fuzz campaign evidence is archived
- deployment evidence pack is archived
- release artifacts are checksummed
