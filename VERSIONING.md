# Versioning Policy

SNARK_LAB uses semantic-version-style release numbers for research-preview releases.

Current version line:

    0.x.y

## Meaning of 0.x.y

Before version 1.0, the repository is a production-grade research prototype but not an audited production cryptographic library.

Breaking API changes may occur between minor versions.

## Patch releases

Patch releases may include:

- documentation fixes
- test additions
- CI fixes
- benchmark reporting improvements
- bug fixes that do not change public APIs

Example:

    v0.2.1

## Minor releases

Minor releases may include:

- new protocol modules
- new CLI commands
- new proof formats
- new SRS tooling
- new benchmark suites
- public API additions
- controlled breaking changes during the 0.x phase

Example:

    v0.3.0

## Major releases

A 1.0 release requires:

- stable public API
- completed threat model
- security proof sketch
- public test vectors
- long-running fuzz campaign
- audited or externally reviewed cryptographic path
- production SRS story
- documented release reproducibility

Until then, releases must be labeled research-preview.

## Release tags

Release tags use:

    vMAJOR.MINOR.PATCH

Example:

    v0.2.0

## Security language

No release before 1.0 may be described as production-secure cryptographic software.

Allowed language:

- production-grade research prototype
- research-preview release
- unaudited protocol engineering artifact

Forbidden language:

- audited
- production-secure
- mainnet-ready
- safe for custody
- safe for consensus-critical use
