# SNARK_LAB v0.2.0-rc.1

This is the first SNARK_LAB release candidate.

## Release class

release-candidate

## Security status

This release is not production-secure.

It is intended for:

- protocol review
- reproducibility review
- artifact review
- external audit preparation
- deployment-process validation

It is not intended for:

- production funds
- custody
- mainnet systems
- consensus-critical infrastructure
- security-critical deployment

## Highlights

- Sumcheck, Zerocheck, and PermCheck protocol labs
- IPA multilinear polynomial commitment path
- IPA proof/opening/SRS codec hardening
- SRS provenance and loader tooling
- Public test vectors
- Negative proof fixtures
- Randomized IPA roundtrip tests
- Fuzz target compile checks
- Deployment evidence pack process
- Audit readiness packet
- Release candidate evidence summary
- Browser visualizer with System and IPA flow tabs
- Production readiness gate with release checklist

## Evidence

Primary evidence files:

    release-candidates/LATEST.md
    release-candidates/LATEST.json

Release notes:

    release/v0.2.0-rc.1.md

Audit packet:

    audits/packet/README.md

Deployment evidence process:

    deployment/README.md
    docs/deployment-evidence-pack.md

SRS ceremony process:

    ceremony/README.md
    docs/production-srs-ceremony-spec.md

## Checksums

Attach the generated SHA256SUMS file from:

    dist/releases/v0.2.0-rc.1/SHA256SUMS

## Remaining before production-secure release

- External cryptographic audit
- Side-channel review
- Production SRS artifact and digest
- Production ceremony transcript
- Long fuzz campaign archive
- Final production deployment approval
