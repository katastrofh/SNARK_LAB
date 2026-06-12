# Sumcheck Example

Sumcheck proves that a multilinear polynomial has a claimed sum over the Boolean hypercube.

## What to inspect

Look for:

- round messages
- challenge derivation
- final evaluation check
- rejection of tampered claims
- transcript binding

## Why it matters

Sumcheck is the backbone reduction used by many SNARK systems.

It turns a large sum claim into a sequence of smaller univariate checks.

## Suggested review commands

    cargo test -p sumcheck

## Things to verify manually

- The verifier rejects a wrong claimed sum.
- The verifier rejects tampered round messages.
- The final evaluation is bound to the transcript.
- Serialization rejects malformed proofs.
