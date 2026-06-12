# Zerocheck Example

Zerocheck verifies that a constraint table is zero on the Boolean hypercube.

## What to inspect

Look for:

- constraint table construction
- oracle binding before challenge derivation
- rejection of nonzero tables
- connection to Sumcheck-style reductions

## Suggested review commands

    cargo test -p zerocheck

## Things to verify manually

- A zero table verifies.
- A nonzero table is rejected.
- The constraint oracle is bound before mixing challenges.
