# Nightly Fuzz Smoke Evidence

This document records the policy for SNARK_LAB nightly fuzz smoke evidence.

## Scope

This branch records a successful short smoke run for:

    ipa_proof_decode

## Evidence files

Evidence is stored in:

    fuzz/smoke-evidence/v0.2.0-rc.1/

Files:

- ipa_proof_decode_smoke.md
- manifest.json
- ipa_proof_decode_smoke_tail.log

## Boundary

This is smoke evidence only.

It does not claim:

- production security
- full parser exhaustion
- full fuzz campaign completion
- external audit completion
- side-channel review completion

## Production requirement

A production-secure release still requires long fuzz campaign evidence across all fuzz targets, with archived logs, crash triage, regression tests, and checksums.
