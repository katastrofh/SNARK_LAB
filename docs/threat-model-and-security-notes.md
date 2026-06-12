# Threat Model and Security Notes

This document defines the security boundary of SNARK_LAB.

SNARK_LAB is serious protocol infrastructure and a production-grade research prototype. It is not audited and must not be used for production funds, mainnet deployments, custody, consensus-critical infrastructure, or security-critical systems.

## Scope

This threat model covers:

- Rust protocol crates
- Fiat-Shamir transcript binding
- Sumcheck, Zerocheck, and PermCheck implementations
- IPA polynomial-commitment infrastructure
- IPA proof serialization
- IPA SRS serialization and validation
- CLI validation tooling
- Browser visualizer boundaries
- CI and repository hardening

This document does not claim security for external deployments, downstream integrations, modified forks, or unaudited releases.

## Assets

Security-relevant assets include:

- Fiat-Shamir transcript binding
- Sumcheck proof soundness under the implemented oracle model
- Zerocheck reduction ordering
- PermCheck product and rational fingerprint ordering
- IPA commitment equation enforcement
- IPA opening proof verification
- IPA proof serialization and decoding
- IPA SRS file validation
- SRS provenance metadata
- malformed-input rejection
- explicit non-production boundaries

## Adversary model

The adversary may:

- choose malicious public statements
- choose malformed proof bytes
- choose malformed SRS files
- mutate transcript exports
- mutate CLI input paths
- mutate serialized IPA openings
- mutate serialized IPA SRS files
- provide invalid curve-point encodings
- provide identity points
- attempt duplicate generator material
- attempt wrong generator counts
- attempt wrong transcript labels
- attempt wrong opening points
- attempt wrong public commitments
- attempt tampered IPA round commitments
- attempt tampered final IPA scalars
- attempt denominator-pole inputs for rational PermCheck

The adversary may not:

- break standard cryptographic assumptions of the selected curve and field
- forge collision-resistant hashes
- break Fiat-Shamir assumptions without finding transcript collisions
- bypass Rust type safety through unsafe code in this repository, because unsafe Rust is rejected by the production gate

## Trust assumptions

The repository currently assumes:

- Arkworks field and curve implementations are correct
- Merlin transcript implementation is correct
- SHA-256 implementation is correct
- Rust compiler and standard library are correct
- CI executes the configured checks honestly
- externally supplied production SRS material was generated correctly outside this repository

The repository does not currently implement a production SRS ceremony or hash-to-curve generator derivation pipeline.

## Security properties targeted

### Fiat-Shamir ordering

Every protocol challenge must be derived only after the relevant public statement and previous prover messages are transcript-bound.

Changing an earlier statement or message must change later challenges.

### Sumcheck

The Sumcheck implementation targets transcript-bound verification for multilinear tables under the implemented oracle model.

Malformed rounds, wrong claims, and tampered final evaluations must be rejected.

### Zerocheck

The constraint oracle must be bound before the equality-mixing challenge is sampled.

A nonzero constraint table must be rejected.

### PermCheck

The tagged columns must be transcript-bound before beta and gamma are derived.

Rational PermCheck must fail explicitly on denominator poles instead of silently accepting invalid arithmetic.

### IPA commitment path

The IPA path targets:

- checked curve generator bases
- canonical compressed curve-point serialization
- identity-point rejection
- duplicate-generator rejection
- commitment equation enforcement
- transcript-bound opening statements
- checked L/R reduction-round commitments
- checked vector and generator folding
- prover opening generation
- verifier recursive commitment-relation checking
- final commitment relation checking
- blinded opening support through explicit extension
- negative malformed-proof rejection
- randomized roundtrip verification

### IPA serialization

The byte-facing IPA decoders must reject:

- wrong magic
- truncated input
- trailing bytes
- invalid field encodings
- corrupt inner proofs
- claimed-value mismatches
- malformed SRS files
- SRS digest mismatches

A successful decode is not proof acceptance. Verification remains a separate cryptographic check.

### IPA SRS

Production SRS material must pass:

- format validation
- curve-point decoding
- identity rejection
- duplicate rejection
- generator-count validation
- provenance validation
- canonical basis digest validation

Known-discrete-log test fixture provenance must be rejected by production validation.

## Non-goals

This repository does not currently provide:

- audited production SNARK library status
- production SRS generation
- trusted setup ceremony implementation
- hash-to-curve SRS derivation implementation
- constant-time guarantee
- side-channel audit
- hardware-backed benchmark counters
- formal machine-checked proof
- mainnet deployment guidance
- wallet or custody integration
- consensus-critical deployment readiness

## Browser visualizer boundary

The browser visualizer is educational.

It may use small readable fields and interactive state displays. It must not be treated as the Rust cryptographic implementation.

Educational transcript interchange is isolated in the interchange crate and is namespaced separately from the Rust cryptographic transcript path.

## CLI boundary

The CLI may:

- verify educational transcript fixtures
- run the IPA demo path
- validate IPA SRS files

The CLI must not silently generate production SRS material.

The CLI must fail closed on unsupported curves and malformed files.

## Fuzzing boundary

The fuzz targets exercise parser robustness for:

- IPA opening proof bytes
- IPA integrated opening bytes
- IPA SRS file bytes

Fuzz targets are not proof acceptance tests. They are malformed-input hardening tests.

## Current hardening

The repository currently includes:

- cargo fmt checks
- clippy with warnings denied
- full workspace tests
- fuzz target compile checks
- visualizer production build
- unsafe Rust rejection
- visualizer NaN-footgun rejection
- Linux and macOS CI matrix
- RustSec audit workflow
- npm high-severity audit
- Dependabot configuration
- negative IPA proof fixtures
- randomized IPA roundtrip tests
- CLI SRS integration tests

## Remaining review items

Before any production-security claim, the following must be completed:

- external cryptographic audit
- side-channel review
- constant-time review where relevant
- long-running fuzz campaigns
- dependency audit triage
- formal security proof sketch
- complete benchmark methodology
- real SRS generation or ceremony documentation
- public test vectors
- release reproducibility process

## Classification

Correct current classification:

- production-grade research prototype
- serious protocol engineering infrastructure
- not audited
- not production-secure deployment software
