# All Fuzz Targets Smoke Evidence

This document records successful short nightly smoke runs for all SNARK_LAB fuzz targets.

## Targets

- ipa_proof_decode
- ipa_integrated_opening_decode
- ipa_srs_file_decode

## Evidence files

Evidence is stored under:

    fuzz/smoke-evidence/v0.2.0-rc.1/all-targets/

Files:

- README.md
- manifest.json
- ipa_proof_decode.tail.log
- ipa_integrated_opening_decode.tail.log
- ipa_srs_file_decode.tail.log

## Boundary

This is smoke evidence only.

It does not claim:

- production security
- long fuzz campaign completion
- parser exhaustion
- external audit completion
- side-channel review completion

## Production requirement

A production-secure release still requires long-duration fuzz campaign evidence across all fuzz targets, archived logs, crash triage, regression tests, and checksums.
