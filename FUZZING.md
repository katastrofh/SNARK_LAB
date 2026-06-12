# Fuzzing

This document defines the fuzzing boundary for SNARK_LAB.

## Current fuzz targets

The repository currently includes parser-facing fuzz targets for:

- ipa_proof_decode
- ipa_integrated_opening_decode
- ipa_srs_file_decode

These targets exercise malformed external byte inputs.

## Boundary

Fuzzing is parser hardening.

A successful decode is not proof acceptance. Cryptographic verification remains a separate check.

Fuzzing does not replace:

- formal proof review
- side-channel review
- dependency review
- external cryptographic audit

## Installing cargo-fuzz

Run:

    cargo install cargo-fuzz

## Compile-check all fuzz targets

Run:

    scripts/check-fuzz-targets.sh

This is part of the production gate.

## Run a bounded campaign

Run:

    scripts/run-fuzz-campaign.sh ipa_proof_decode 300
    scripts/run-fuzz-campaign.sh ipa_integrated_opening_decode 300
    scripts/run-fuzz-campaign.sh ipa_srs_file_decode 300

The second argument is the duration in seconds.

## Suggested campaign lengths

For local development:

    60 seconds per target

Before a research-preview release:

    30 minutes per target

Before stronger security claims:

    24 hours per target, across multiple machines if possible

## Campaign logs

Campaign logs are written to:

    fuzz/campaigns/

Keep important campaign summaries, but avoid committing huge raw corpora or crash artifacts unless they are minimized regression cases.

## Crash handling

If fuzzing finds a crash:

1. Minimize the input.
2. Add the minimized input as a regression test.
3. Fix the parser.
4. Re-run the target.
5. Document the fix in CHANGELOG.md.

## Current status

The repository currently guarantees that fuzz targets compile in the production gate.

Long-running campaign artifacts are still required before stronger production-security claims.
