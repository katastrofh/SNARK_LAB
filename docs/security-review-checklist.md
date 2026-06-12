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

## Release checklist and tagging

- [x] Production release checklist added
- [x] Release notes template added
- [x] Release tag preparation script added
- [x] Release checklist checker added
- [ ] Release candidate tag created
- [ ] Release artifacts attached to GitHub release

## Release candidate tag

- [x] Release candidate notes added
- [ ] Annotated release candidate tag created
- [ ] Release candidate tag pushed
- [ ] GitHub release created

## GitHub release artifacts

- [x] GitHub release artifact tooling added
- [x] Release checksum generation added
- [x] GitHub release draft added
- [ ] GitHub pre-release created
- [ ] Release artifacts attached
- [ ] SHA256SUMS attached

## Long fuzz campaign evidence

- [x] Long fuzz campaign evidence process added
- [x] Long fuzz campaign runner added
- [x] Long fuzz campaign manifest template added
- [ ] Real long fuzz campaign completed
- [ ] Long fuzz campaign logs archived
- [ ] Fuzz crash regressions added

## Production deployment guide

- [x] Production deployment guide added
- [x] Operator runbook added
- [x] Deployment decision template added
- [x] Production readiness index added
- [ ] Production deployment approved

## Production SRS artifact policy

- [x] Production SRS placeholder policy added
- [x] Fake production SRS artifact check added
- [x] Production SRS status example added
- [ ] Real production SRS artifact published
- [ ] Real production SRS digest published
- [ ] Real production SRS transcript published

## Fuzz nightly runner

- [x] Nightly fuzz runner policy added
- [x] Runner checks for rustup toolchain availability
- [x] Runner records fuzz toolchain in campaign manifest
- [ ] Nightly smoke campaign completed
- [ ] Long nightly fuzz campaign completed

## Fuzz campaign runner hardening

- [x] Fuzz generated artifacts ignored
- [x] Failed fuzz run diagnostic tail added
- [x] Failed fuzz runs explicitly not evidence
- [ ] Failed smoke run triaged
- [ ] Successful smoke campaign archived

## IPA fuzz runtime

- [x] Nightly rust-src fuzz preflight added
- [x] IPA fuzz runtime requirements documented
- [ ] IPA fuzz smoke run completed
- [ ] IPA fuzz long campaign completed

## IPA proof decoder fuzz regression

- [x] IPA proof decoder capacity-overflow fuzz regression added
- [x] IPA proof variables bound checked before allocation
- [x] IPA proof round count bound checked before allocation
- [ ] Full IPA proof decoder fuzz campaign completed

## Nightly fuzz smoke evidence

- [x] IPA proof decoder nightly fuzz smoke run completed
- [x] Smoke evidence manifest added
- [x] Smoke evidence marked non-production
- [ ] All fuzz targets smoke-run cleanly
- [ ] Long fuzz campaign completed

## All fuzz targets smoke evidence

- [x] All fuzz targets smoke-run cleanly
- [x] All-target smoke evidence manifest added
- [x] All-target smoke evidence marked non-production
- [ ] Long fuzz campaign completed
- [ ] Crash corpus regression suite completed

## Fuzz crash regression suite

- [x] Fuzz crash regression suite added
- [x] IPA proof decoder capacity-overflow crash regression added
- [x] Regression metadata validation added
- [x] Regression test checks no panic
- [ ] More crash corpus cases added as fuzzing discovers them

## README public boundary

- [x] README public security boundary checked
- [x] README evidence table added
- [x] README quickstart added
- [x] README release-candidate status added
- [x] Visualizer screenshot assets added

## Visualizer screenshot assets

- [x] Visualizer screenshot assets added
- [x] Visualizer screenshot checker added
- [x] README references screenshot assets

## Repository topic and badge polish

- [x] Repository discovery metadata added
- [x] README badges added
- [x] Badge security-boundary checker added
- [x] Suggested GitHub topics documented

## GitHub Release page finalization

- [x] GitHub Release page body added
- [x] Release asset list included
- [x] Checksum verification command included
- [x] Non-production security boundary included
- [x] GitHub Release published manually

## Manual GitHub Release publication evidence

- [x] GitHub Release publication evidence recorded
- [x] Release asset list validated
- [x] Release URL recorded
- [x] Publication evidence marked non-production
