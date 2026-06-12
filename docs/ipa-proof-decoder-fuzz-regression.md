# IPA Proof Decoder Fuzz Regression

A cargo-fuzz run found that malformed IPA proof bytes could trigger a capacity overflow panic in the proof decoder.

## Root cause

The decoder accepted unbounded `variables` and `round_count` values from the byte stream and allocated `Vec::with_capacity(round_count)` before validating the round count against a safe bound.

## Fix

The decoder now checks:

- decoded variable count is bounded
- decoded round count is bounded
- decoded round count matches decoded variable count
- malformed count fields return `LengthOverflow` or proof-shape errors instead of panicking

## Regression

The fuzz-generated crashing input is now a unit test:

    rejects_fuzzed_oversized_round_count_without_panic

## Boundary

This fix hardens byte parsing.

It does not change proof acceptance semantics. A successful decode is still not proof acceptance; cryptographic verification remains separate.
