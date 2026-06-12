# Release Process

This directory contains release-checklist and tagging material for SNARK_LAB.

## Release classes

SNARK_LAB supports three release classes:

- research-preview
- release-candidate
- production-secure

## Current allowed release class

Until external audit, side-channel review, and production SRS evidence are complete, the allowed status is:

    release-candidate

or:

    research-preview

Do not publish a production-secure release until the production criteria are complete.

## Required before tagging

Run:

    scripts/check-release-checklist.sh
    scripts/check-production-ready.sh

Then prepare a tag with:

    scripts/prepare-release-tag.sh v0.2.0-rc.1

To actually create the tag, run:

    scripts/prepare-release-tag.sh v0.2.0-rc.1 --create
