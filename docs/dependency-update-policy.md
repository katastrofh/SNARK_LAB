# Dependency Update Policy

This document defines how dependency updates are handled in SNARK_LAB.

## Rule 1: do not merge failing dependency PRs

A dependency PR must not be merged unless all required checks pass.

Required checks include:

- production-readiness
- Rust Linux CI
- Rust macOS CI
- visualizer build
- RustSec audit
- npm audit where applicable

## Rule 2: cryptographic dependency updates are coordinated

Cryptographic dependencies must not be upgraded one crate at a time when their traits or type identities are shared across the codebase.

This especially applies to Arkworks crates:

- ark-ff
- ark-ec
- ark-serialize
- ark-bls12-381
- ark-poly

## Current Arkworks policy

SNARK_LAB currently pins the Arkworks stack to the 0.5 version line.

Dependabot is configured to ignore minor and major Arkworks version bumps.

A future upgrade to Arkworks 0.6 must happen in a dedicated branch:

    arkworks-0-6-coordinated-upgrade

That branch must upgrade all Arkworks crates together and run the full production gate.

## Why one-by-one Arkworks upgrades are unsafe

Rust treats traits from different crate versions as different traits.

For example, ark-ff 0.5 and ark-ff 0.6 define different PrimeField traits.

Mixing ark-bls12-381 0.5 with ark-ff 0.6 can cause field types to fail trait bounds even when the names look identical.

## Acceptable automatic updates

Automatic updates may be accepted when all checks pass for:

- patch-level non-cryptographic Rust dependencies
- patch-level npm dependencies
- GitHub Actions updates

## Manual review required

Manual review is required for:

- cryptographic dependencies
- proof-system dependencies
- parser or serialization dependencies
- transcript dependencies
- build-system changes
- any dependency update that changes Cargo.lock substantially

## Rejection criteria

Reject or close a dependency PR if:

- it fails production-readiness
- it introduces mixed cryptographic crate versions
- it changes public proof formats unexpectedly
- it changes SRS serialization unexpectedly
- it creates duplicate GitHub Actions runs
- it requires new unsafe code
- it weakens parser rejection behavior
