# Security Review Checklist

This checklist tracks what must hold before SNARK_LAB can make stronger security claims.

## Build and CI

- [x] Local production gate
- [x] Git pre-push production gate
- [x] GitHub Actions production gate
- [x] Linux CI
- [x] macOS CI
- [x] Rust formatting check
- [x] Clippy with warnings denied
- [x] Full workspace tests
- [x] Visualizer production build
- [x] Fuzz target compile check
- [x] RustSec audit workflow
- [x] npm audit workflow
- [x] Dependabot

## Rust safety

- [x] Unsafe Rust rejected by production gate
- [x] Crate roots forbid unsafe code where added
- [ ] Review all dependencies for unsafe-heavy internals
- [x] Document dependency trust assumptions

## Serialization

- [x] IPA proof codec rejects wrong magic
- [x] IPA proof codec rejects truncation
- [x] IPA proof codec rejects trailing bytes
- [x] IPA integrated opening codec rejects claimed-value mismatch
- [x] IPA SRS loader rejects wrong magic
- [x] IPA SRS loader rejects truncation
- [x] IPA SRS loader rejects trailing bytes
- [x] IPA SRS loader rejects digest mismatch
- [x] Fuzz targets for proof, opening, and SRS decoders
- [x] Fuzz target campaign runner
- [ ] Long-running fuzz campaign artifacts
- [ ] Corpus minimization and regression corpus

## IPA PCS

- [x] Commitment equation implemented
- [x] Opening statement binding implemented
- [x] Reduction-round folding implemented
- [x] L/R commitments implemented
- [x] Prover opening loop implemented
- [x] Verifier opening loop implemented
- [x] Blinded opening extension implemented
- [x] Integrated commit, open, and verify API implemented
- [x] Negative malformed-proof fixtures
- [x] Randomized roundtrip tests
- [x] Independent reference implementation comparison
- [ ] Public test vectors
- [ ] Formal proof sketch

## SRS

- [x] SRS provenance metadata
- [x] Canonical SRS digest
- [x] SRS file loader
- [x] CLI SRS validation
- [x] Known-discrete-log fixture provenance rejected
- [ ] Production SRS ceremony document
- [ ] Hash-to-curve derivation implementation or external ceremony evidence
- [ ] Public SRS artifact process

## Side channels

- [x] Side-channel boundary documented
- [x] Production deployment evidence checklist documented

- [ ] Constant-time review
- [ ] Secret-dependent branching review
- [ ] Secret-dependent memory-access review
- [ ] Benchmark noise and timing methodology
- [ ] Document what is public versus secret in benchmarks

## Documentation

- [x] README production boundary
- [x] SECURITY.md
- [x] Threat model
- [x] Security review checklist
- [ ] Security proof sketch
- [ ] Release process
- [ ] Public audit notes

## SRS ceremony

- [x] Production SRS ceremony specification documented
- [x] SRS ceremony manifest example added
- [x] SRS ceremony manifest verifier added
- [ ] Real production SRS artifact published
- [ ] Real production SRS digest published
- [ ] Real ceremony transcript published
- [ ] External review of SRS ceremony completed

## Deployment evidence

- [x] Deployment evidence pack process documented
- [x] Deployment evidence collector added
- [x] Deployment attestation template added
- [ ] Real release-candidate evidence pack generated
- [ ] Production evidence pack archived with release

## Audit readiness

- [x] Audit readiness packet added
- [x] Audit scope documented
- [x] Audit finding template added
- [x] Remediation log added
- [x] Audit triage policy added
- [ ] External audit completed
- [ ] Critical/high findings resolved
- [ ] Final audit report linked

## Release candidate evidence

- [x] Release-candidate evidence summary tooling added
- [ ] Release-candidate evidence run generated from clean release commit
- [ ] Release-candidate evidence archived with GitHub release
