# IPA Codec Fuzz Targets

This branch adds fuzz targets for byte-facing IPA parsers.

## Targets

    ipa_proof_decode
    ipa_integrated_opening_decode
    ipa_srs_file_decode

## Scope

The targets exercise:

    IPA opening proof decoder
    IPA integrated opening decoder
    IPA SRS file decoder

## Production boundary

The fuzz targets must never panic on malformed external input.

A successful decode is not considered proof acceptance. Verification remains a separate cryptographic check.

## Build check

The local production gate compiles all fuzz targets with:

    cargo check --locked --manifest-path fuzz/Cargo.toml --bins

## Running fuzz campaigns

Install cargo-fuzz:

    cargo install cargo-fuzz

Run a target:

    cargo fuzz run ipa_proof_decode
    cargo fuzz run ipa_integrated_opening_decode
    cargo fuzz run ipa_srs_file_decode

The fuzz package is isolated in `fuzz/` as its own workspace so it does not pollute the main workspace member list.
