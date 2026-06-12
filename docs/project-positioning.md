# Project Positioning

SNARK_LAB is a Rust protocol lab for studying, testing, and explaining core SNARK building blocks.

It focuses on protocol clarity, reproducible checks, evidence discipline, and implementation hardening around:

- Sumcheck
- Zerocheck
- PermCheck
- multilinear polynomials
- IPA polynomial commitments
- proof serialization
- fuzzing
- release evidence
- educational visualization

## What this project is

SNARK_LAB is:

- a research prototype
- a protocol-learning lab
- an implementation study vehicle
- a reproducibility-oriented codebase
- a place to connect math, code, tests, fuzzing, and release artifacts

## What this project is not

SNARK_LAB is not:

- audited production cryptographic software
- a custody system
- a mainnet deployment
- a replacement for mature proof-system libraries
- a production SRS ceremony output
- a side-channel-reviewed implementation

## Why it is useful

The project is useful because it keeps several usually-separated layers visible at once:

- protocol math
- Rust implementation
- verifier rejection paths
- proof serialization boundaries
- fuzzing evidence
- release artifact evidence
- visual explanations

This makes the repository suitable for protocol study, code review practice, research prototyping, and educational demonstrations.

## Current maturity

The current release candidate has:

- deterministic test vectors
- negative proof fixtures
- randomized roundtrip tests
- fuzz smoke evidence
- fuzz crash regression evidence
- release artifact checksums
- GitHub Release publication evidence
- production-gate scripts
- visualizer screenshot assets

The project still requires external review before any production-security claim.

## Security boundary

All public claims should preserve this boundary:

    Research prototype. Not audited production-secure software.

Do not describe the project as suitable for production deployment, safe for custody use, reviewed by external auditors, or ready for mainnet.
