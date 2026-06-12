# Production Deployment Guide

This document defines the deployment boundary for SNARK_LAB.

## Current deployment status

Current allowed status:

    release-candidate

Current forbidden status:

    production-secure

SNARK_LAB must not be deployed for production funds, custody, mainnet systems, consensus-critical infrastructure, or security-critical deployments until the production-secure blockers are complete.

## What can be deployed today

The current release-candidate can be deployed for:

- protocol review
- artifact review
- internal engineering review
- reproducibility checks
- audit preparation
- demonstration of the visualizer
- non-production CLI validation
- research artifact evaluation

## What must not be deployed today

Do not deploy the current release candidate for:

- production funds
- custody
- mainnet proofs
- consensus-critical verification
- private witness production workflows
- production SRS ceremony claims
- audited-security claims

## Required pre-deployment evidence

Before any production deployment, collect:

- release-candidate evidence summary
- deployment evidence pack
- production gate output
- release artifact checksums
- SRS manifest and digest
- audit status
- side-channel review status
- fuzz campaign status
- release notes
- rollback plan

## Release artifact verification

Given a release tag, generate artifacts:

    scripts/build-github-release-artifacts.sh v0.2.0-rc.1

Verify checksums:

    cd dist/releases/v0.2.0-rc.1
    sha256sum -c SHA256SUMS

## Production gate

Before deployment, run:

    scripts/check-production-ready.sh

A failed production gate blocks deployment.

## SRS handling

Production deployment requires a real SRS or transparent public-parameter artifact.

Required SRS evidence:

- final SRS artifact
- SHA-256 digest
- manifest
- transcript digest
- verifier command output
- ceremony or derivation description
- external review status

The example manifest in this repository is not production SRS evidence.

## Audit handling

Production deployment requires external review.

Required audit evidence:

- auditor identity
- scope
- commit under review
- report
- findings
- remediation log
- final status

No production-secure claim may be made with open critical or high findings.

## Side-channel handling

Production deployment requires side-channel review.

Required side-channel evidence:

- public/secret data classification
- secret-dependent branch review
- secret-dependent memory-access review
- RNG review
- dependency arithmetic review
- logging review

## Fuzzing handling

Production deployment requires archived fuzz evidence.

Required fuzz evidence:

- campaign duration
- target names
- logs
- crashes
- timeouts
- regressions added
- SHA-256 manifest

Compile-only fuzz checks are not sufficient for production-secure status.

## Runtime monitoring

A production deployment plan must define:

- version deployed
- release tag
- artifact digests
- operator identity
- deployment timestamp
- expected input format
- rejected proof monitoring
- parser failure monitoring
- panic monitoring
- resource usage monitoring
- rollback trigger

## Rollback

Rollback requires:

- previous release tag
- previous artifact digest
- previous deployment evidence
- rollback operator
- rollback reason
- post-rollback verification command

## Deployment decision

Allowed deployment decision values:

- reject
- internal-review-only
- release-candidate-demo
- production-secure-approved

The current repository supports:

    release-candidate-demo

It does not yet support:

    production-secure-approved
