# IPA SRS Tooling Tests

This branch adds end-to-end tests for the IPA SRS CLI validation path.

## What is tested

The tests run the actual `snark-lab-cli` binary and verify:

    valid production SRS files are accepted
    explicit curve selection works
    default curve selection works
    unsupported curves fail before file access
    missing SRS files fail cleanly
    wrong format magic fails closed
    truncated SRS files fail closed
    trailing bytes fail closed

## Production boundary

These tests do not generate production SRS material.

They create temporary test fixtures that are immediately validated through the same fail-closed SRS provenance and loader path used by the CLI.

## Command covered

    snark-lab-cli ipa-srs-validate [--curve bls12-381-g1] <path.srs>
