# Long Fuzz Campaign Notes

This branch adds reproducible fuzz-campaign infrastructure.

## Added

- FUZZING.md
- scripts/check-fuzz-targets.sh
- scripts/run-fuzz-campaign.sh
- fuzz/campaigns/README.md

## Targets

Current fuzz targets:

- ipa_proof_decode
- ipa_integrated_opening_decode
- ipa_srs_file_decode

## Production gate

The production gate compile-checks all fuzz targets through:

    scripts/check-fuzz-targets.sh

## Long campaign boundary

This branch does not claim that a long campaign has already been completed.

It adds the process required to run and document one.

## Why this matters

The repository already rejects malformed proof and SRS encodings through unit tests. Fuzzing adds broader malformed-input coverage against byte-facing parsers.

## Before stronger production claims

Before stronger security claims, run and document:

- 24 hours on ipa_proof_decode
- 24 hours on ipa_integrated_opening_decode
- 24 hours on ipa_srs_file_decode
- minimized regression tests for any discovered crashes
