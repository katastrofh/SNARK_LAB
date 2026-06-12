# Audit Packet Index

This is the recommended starting point for reviewers.

## Primary files

- audits/scope.md
- audits/triage-policy.md
- audits/remediation-log.md
- audits/audit-status.example.json
- audits/templates/finding-template.md
- audits/templates/audit-request-template.md

## Security documents

- SECURITY.md
- docs/threat-model.md
- docs/security-proof-sketch.md
- docs/side-channel-boundary-notes.md
- docs/production-deployment-evidence.md
- docs/production-srs-ceremony-spec.md
- docs/deployment-evidence-pack.md
- docs/dependency-update-policy.md

## Reproducibility and evidence

- test-vectors/README.md
- FUZZING.md
- ceremony/README.md
- deployment/README.md
- scripts/check-production-ready.sh
- scripts/collect-deployment-evidence.sh

## Recommended reviewer commands

Run:

    scripts/check-production-ready.sh

Then generate a local evidence pack:

    scripts/collect-deployment-evidence.sh

Then inspect:

    deployment/evidence/<timestamp>/SUMMARY.md
    deployment/evidence/<timestamp>/attestation.json

## Current claim

Current claim:

    production-grade research prototype

Not yet claimed:

    externally audited
    production-secure
    production SRS ceremony completed
