# Reference Implementation Comparison

This branch adds an independent reference implementation for selected protocol algebra.

## Purpose

The reference implementation is intentionally slow, direct, and readable.

It is used to compare production code against a separate implementation of:

- multilinear evaluation
- IPA evaluation-basis construction
- IPA evaluation inner product
- IPA polynomial-vector folding
- IPA evaluation-vector folding
- transparent oracle opening and verification

## Why this matters

Unit tests often test local behavior against itself.

Reference comparison tests reduce that risk by checking production functions against an independently written implementation of the same algebra.

## Current scope

The reference comparison covers deterministic tables up to 6 variables.

This is enough to exercise the ordering conventions and folding identities without making CI slow.

## Boundary

This is not a formal proof and not an audit.

It is an engineering hardening layer for regression detection and reviewer confidence.
