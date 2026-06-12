# IPA Fuzz Target Runtime Fix

This document records the runtime requirements for running IPA fuzz targets.

## Problem

The fuzz targets compile on stable during the production gate.

Actual cargo-fuzz execution requires nightly Rust because cargo-fuzz uses sanitizer options. Some sanitizer builds also require the nightly rust-src component.

## Required setup

Run:

    rustup toolchain install nightly
    rustup component add rust-src --toolchain nightly
    cargo install cargo-fuzz

## Smoke command

Run:

    cd fuzz
    cargo +nightly fuzz run ipa_proof_decode -- -max_total_time=10

## Long campaign

Run from repository root:

    FUZZ_SECONDS_PER_TARGET=300 scripts/run-long-fuzz-campaign.sh

## Evidence boundary

A failed fuzz run is not fuzz evidence.

A successful smoke run is only smoke evidence.

A production fuzz campaign requires long duration, archived logs, crash triage, regressions, and checksums.
