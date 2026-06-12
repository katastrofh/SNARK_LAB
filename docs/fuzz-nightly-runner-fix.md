# Fuzz Nightly Runner Fix

SNARK_LAB fuzz targets compile during the production gate on the stable toolchain.

Actual `cargo fuzz run` campaigns require nightly because cargo-fuzz uses sanitizer options such as:

    -Zsanitizer=address

Stable Rust rejects `-Z` options.

## Policy

The long fuzz campaign runner uses:

    FUZZ_TOOLCHAIN=nightly

by default.

## Install nightly

Run:

    rustup toolchain install nightly

## Install cargo-fuzz

Run:

    cargo install cargo-fuzz

## Smoke run

Run:

    FUZZ_SECONDS_PER_TARGET=10 scripts/run-long-fuzz-campaign.sh

## Use another toolchain

Run:

    FUZZ_TOOLCHAIN=nightly-YYYY-MM-DD FUZZ_SECONDS_PER_TARGET=60 scripts/run-long-fuzz-campaign.sh

## Boundary

Stable CI compiles fuzz targets.

Nightly campaign runs are separate evidence-generation runs and should be archived as release evidence.
