# IPA Randomized Roundtrip Tests

This branch adds reproducible randomized tests for the integrated IPA backend.

## What is randomized

The tests generate random:

    multilinear evaluation tables
    opening points
    commitment blinding scalars

For each case, the backend runs:

    commit
    open
    encode
    decode
    verify

## Why the RNG is seeded

The tests use seeded RNGs because CI failures must be reproducible.

This is test-only randomness. It is not used for production blinding, SRS generation, or generator derivation.

## Negative randomized cases

The test suite rejects:

    tampered final scalar
    wrong opening point
    wrong public commitment
    wrong transcript label

## Production boundary

The generator material in this module is test fixture material only.

Production generator derivation or SRS loading is still a separate item.
