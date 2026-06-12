# Long Fuzz Campaign Evidence

This document describes how SNARK_LAB records long fuzz campaign evidence.

## Added

- fuzz/campaigns/README.md
- fuzz/campaigns/TEMPLATE.md
- fuzz/campaigns/long-fuzz-campaign-manifest.example.json
- scripts/run-long-fuzz-campaign.sh
- scripts/check-long-fuzz-campaign-evidence.sh

## Targets

Current fuzz campaign targets:

- ipa_proof_decode
- ipa_integrated_opening_decode
- ipa_srs_file_decode

## Run a smoke campaign

Run:

    FUZZ_SECONDS_PER_TARGET=30 scripts/run-long-fuzz-campaign.sh

## Run a serious campaign

Run each target for hours, not seconds.

Example:

    FUZZ_SECONDS_PER_TARGET=21600 scripts/run-long-fuzz-campaign.sh

This means six hours per target.

## Evidence output

Generated campaign evidence is written to:

    fuzz/campaigns/<timestamp>/

The directory contains:

- SUMMARY.md
- manifest.json
- one log per target
- artifacts directory per target
- SHA256SUMS

## Production boundary

Fuzz campaign evidence improves parser-hardening confidence.

It does not prove production security.

Production-secure status still requires:

- external audit
- side-channel review
- production SRS evidence
- release evidence
- production deployment approval

## Nightly requirement

Actual fuzz campaign execution requires nightly Rust because cargo-fuzz uses sanitizer `-Z` flags.

Install nightly with:

    rustup toolchain install nightly

The runner defaults to:

    FUZZ_TOOLCHAIN=nightly
