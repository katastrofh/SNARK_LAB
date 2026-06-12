# IPA Negative Proof Fixtures

This document records the malformed-proof rejection fixtures for the integrated IPA backend.

## Negative cases

The test module rejects:

    wrong public commitment
    wrong opening point
    wrong transcript label
    tampered claimed value
    tampered final scalar
    tampered round commitment bytes
    wrong padding generator material
    wrong verifier key size
    corrupt encoded opening

## Production boundary

These are fixed malformed-proof tests for the current integrated backend.

They do not replace fuzzing. They are the stable regression suite that must pass before fuzzing and larger randomized tests are added.

## Security rule

Malformed inputs must fail closed.

The verifier must never accept a proof because the proof shape merely parses.
