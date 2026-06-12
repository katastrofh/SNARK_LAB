# Post-Freeze Maintenance Policy

This policy describes how to maintain SNARK_LAB after the release-candidate freeze.

## Default mode

The default mode after freeze is review and maintenance.

Avoid large unrelated feature branches until external reviewers have had time to inspect the project.

## Acceptable changes

Acceptable post-freeze changes:

- fix broken commands
- fix broken links
- improve unclear docs
- add reviewer-requested explanation
- add benchmark reports
- add fuzz regressions
- add issue templates
- add contribution guidance
- patch release metadata
- update evidence from real review

## Changes requiring justification

The following should include a clear reason in the PR or commit message:

- new protocol component
- new backend
- major visualizer redesign
- large dependency upgrade
- large API change
- benchmark methodology change

## Release candidate policy

Use new release-candidate tags only when the current main branch contains meaningful reviewer-facing improvements or evidence updates.

## Boundary

Maintenance work should preserve the repository's research-prototype positioning.

Do not turn evidence documents into claims of deployment suitability.
