# Audit Scope

## Repository

SNARK_LAB

## Target status

Production-grade SNARK research prototype moving toward production deployment.

## In scope

### Rust protocol crates

- crates/field
- crates/multilinear
- crates/transcript
- crates/oracle
- crates/sumcheck
- crates/zerocheck
- crates/permcheck
- crates/interchange
- crates/cli

### Protocol components

- multilinear evaluation
- equality-weight construction
- Fiat-Shamir transcript binding
- Sumcheck protocol flow
- Zerocheck protocol flow
- PermCheck protocol flow
- IPA polynomial commitment opening path
- IPA folding rounds
- proof/opening/SRS codecs
- SRS provenance and loader
- public test vectors
- reference implementation comparison tests

### Security-relevant tooling

- scripts/check-production-ready.sh
- scripts/check-test-vectors.sh
- scripts/check-fuzz-targets.sh
- scripts/check-srs-ceremony-spec.sh
- scripts/collect-deployment-evidence.sh
- scripts/verify-srs-ceremony-manifest.py

### Byte-facing parsers

- IPA proof decoding
- integrated IPA opening decoding
- SRS file decoding
- interchange serialization/deserialization

### Documentation in scope

- SECURITY.md
- FUZZING.md
- RELEASE.md
- VERSIONING.md
- docs/security-proof-sketch.md
- docs/threat-model.md
- docs/side-channel-boundary-notes.md
- docs/production-deployment-evidence.md
- docs/production-srs-ceremony-spec.md
- docs/deployment-evidence-pack.md
- docs/dependency-update-policy.md

## Out of scope unless explicitly added

- browser visualizer as production software
- browser visualizer side-channel properties
- operating-system hardening
- hardware side channels
- deployment cloud configuration
- production key custody
- real user-data processing
- economic/incentive security
- consensus-layer assumptions
- non-Arkworks cryptographic backends
- uncommitted local scripts
- generated evidence packs not supplied to auditors

## Explicitly not yet claimed

The following are not yet claimed unless real evidence is supplied:

- external audit completed
- side-channel audit completed
- production SRS ceremony completed
- production-secure deployment approved
- mainnet/custody/consensus-critical safety
