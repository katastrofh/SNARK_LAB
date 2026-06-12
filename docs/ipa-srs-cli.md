# IPA SRS CLI

The CLI now exposes:

    snark-lab-cli ipa-srs-validate [--curve bls12-381-g1] <path.srs>

## Purpose

This command validates an IPA SRS file from disk using the canonical SRS loader.

## Validation path

The command performs:

    file read
    canonical SRS decode
    curve-point decode
    identity rejection
    duplicate rejection
    generator-count validation
    provenance validation
    canonical basis digest validation

## Production rule

The command accepts only production-valid SRS files.

It does not generate SRS material.

It does not accept known-discrete-log test fixture provenance.

## Supported curve

Currently supported:

    bls12-381-g1

Unsupported curves fail before file access.

## Example

    cargo run -p snark-lab-cli -- ipa-srs-validate --curve bls12-381-g1 ./srs/ipa-g1.srs
