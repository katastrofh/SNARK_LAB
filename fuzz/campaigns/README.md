# Fuzz Campaign Evidence

This directory describes long-running fuzz campaign evidence for SNARK_LAB.

## Current status

The repository has fuzz targets and compile checks.

A real long-running fuzz campaign must be run separately and archived as release evidence.

## Targets

Current fuzz targets:

- ipa_proof_decode
- ipa_integrated_opening_decode
- ipa_srs_file_decode

## Generate a campaign

Run:

    scripts/run-long-fuzz-campaign.sh

For a short local smoke run:

    FUZZ_SECONDS_PER_TARGET=30 scripts/run-long-fuzz-campaign.sh

For a serious campaign, use hours per target, not seconds.

## Evidence policy

Generated campaign logs are ignored by default.

A production release should archive:

- campaign manifest
- target logs
- duration per target
- machine/environment
- crashes found
- minimized artifacts
- regression tests created from crashes
- final status

## Nightly requirement

Actual fuzz campaign execution requires nightly Rust because cargo-fuzz uses sanitizer `-Z` flags.

Install nightly with:

    rustup toolchain install nightly

The runner defaults to:

    FUZZ_TOOLCHAIN=nightly
