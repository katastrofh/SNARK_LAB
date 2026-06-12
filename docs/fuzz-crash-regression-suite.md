# Fuzz Crash Regression Suite

This document records the SNARK_LAB policy for converting fuzz-discovered crashes into permanent regression tests.

## Added

- fuzz/regressions/README.md
- fuzz/regressions/ipa_proof_decode/capacity-overflow-20260612.json
- crates/oracle/tests/ipa_fuzz_crash_regressions.rs
- scripts/check-fuzz-crash-regressions.sh

## First regression

Target:

    ipa_proof_decode

Bug class:

    capacity-overflow-panic

Expected fixed behavior:

    decode-error-no-panic

Expected error:

    LengthOverflow

## Why this matters

The fuzzer discovered malformed IPA proof bytes with huge count fields that previously caused allocation capacity overflow.

The decoder must reject malformed bytes before dangerous allocation.

## Boundary

Regression tests prevent known crashes from reappearing.

They do not prove production security and do not replace long fuzz campaigns, external audit, side-channel review, or production SRS evidence.
