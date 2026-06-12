# Fuzz Crash Regression Suite

This directory records fuzz-discovered crashes that have been converted into regression tests.

## Policy

Every confirmed fuzz crash should produce:

- a regression metadata file
- a stable unit or integration test
- a short root-cause note
- an expected failure mode
- a confirmation that the malformed input no longer panics

## Current regressions

- ipa_proof_decode capacity-overflow malformed count regression

## Boundary

Regression corpus files are not production security proofs.

They prevent known parser crashes from reappearing.
