# PermCheck Example

PermCheck checks that two tagged sequences match under a claimed permutation relation.

## What to inspect

Look for:

- tagged values
- permutation identity
- rational check structure
- denominator pole handling
- mutation rejection tests

## Suggested review commands

    cargo test -p permcheck

## Things to verify manually

- Matching tagged permutations verify.
- Mutated data is rejected.
- Denominator poles are handled explicitly.
