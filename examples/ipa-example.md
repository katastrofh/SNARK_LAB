# IPA Polynomial Commitment Example

The IPA path demonstrates commitment, opening, verification, proof serialization, and rejection tests.

## What to inspect

Look for:

- commitment equation checks
- generator basis validation
- evaluation basis construction
- opening statement binding
- recursive reduction rounds
- verifier rejection paths
- proof codec hardening

## Suggested review commands

    cargo test -p snark_lab_oracle ipa
    cargo test -p snark_lab_cli ipa_demo

## Things to verify manually

- Wrong commitments are rejected.
- Wrong opening points are rejected.
- Tampered proof scalars are rejected.
- Corrupt encodings do not verify.
- Fuzz regression inputs return errors instead of panics.
