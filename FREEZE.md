# SNARK_LAB Release-Candidate Freeze

Freeze recorded at:

    2026-06-12T16:06:18Z

Main commit at freeze:

    2d114293eb424db337c0984bbb2d2988b979969d

Short commit:

    2d11429

## Freeze status

The current public research-prototype release-candidate phase is frozen.

This means the repository has enough structure, evidence, documentation, releases, and reviewer-facing material to be shown publicly and reviewed.

## Current release candidate

    v0.2.0-rc.2

## What is frozen

The following are considered complete for the current release-candidate phase:

- core protocol lab structure
- Sumcheck, Zerocheck, PermCheck, and IPA protocol paths
- proof serialization and rejection tests
- public vectors
- fuzz smoke evidence
- fuzz crash regression suite
- SRS provenance and placeholder policy
- release artifacts and checksums
- GitHub Release publication evidence
- visualizer and screenshots
- reviewer onboarding guide
- examples gallery
- paper-style overview
- final repository health report
- project positioning and roadmap

## Allowed post-freeze changes

Post-freeze changes should be limited to:

- bug fixes
- documentation corrections
- reviewer-requested clarifications
- new evidence from actual review
- benchmark report additions
- additional fuzz regression cases
- release-follow-up patches

## Avoid post-freeze scope creep

Do not add unrelated features before collecting external feedback.

The next useful step is review, not endless expansion.

## Boundary

This freeze does not change the security status of the project.

The repository remains a protocol lab and research prototype.

It should not be described as deployment-grade cryptographic infrastructure.
