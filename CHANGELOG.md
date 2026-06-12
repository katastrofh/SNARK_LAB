# Changelog

All notable changes to SNARK_LAB will be tracked here.

The project is currently in research-preview status.

## Unreleased

### Added

- Production SRS ceremony specification and manifest verifier

- Side-channel boundary notes
- Production deployment evidence checklist

- System-level visualizer map

- Visualizer demo polish and direct tab links

- Real IPA opening flow in the visualizer

- Fuzz campaign runner and documentation

- Threat model and security notes
- Security proof sketch
- Security review checklist
- IPA SRS provenance validation
- IPA SRS file loader
- IPA SRS validation CLI
- IPA SRS CLI integration tests
- IPA codec fuzz targets
- IPA benchmark suite
- CI matrix and audit workflow
- Release process and versioning policy

### Changed

- Consolidated GitHub Actions to reduce duplicate workflow runs
- Throttled Dependabot update cadence to reduce PR noise
- Pinned Arkworks dependencies to a consistent 0.5 version line
- Clarified production-grade research prototype boundary

### Security

- Added explicit unaudited status
- Added fail-closed parser and SRS validation documentation
- Added release warning language for non-production use

## v0.2.0 - Research Preview

### Added

- Rust protocol core for Sumcheck, Zerocheck, and PermCheck
- Transparent oracle abstraction
- Browser visualizer
- Educational transcript interchange
- IPA PCS typed implementation path
- Blinded IPA opening path
- IPA proof codec
- Negative proof fixtures
- Randomized IPA roundtrip tests
- Local production gate
- Git pre-push production gate
