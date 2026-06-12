# IPA Blinded Prover/Verifier Path

This document connects the blinding extension to the real IPA prover and verifier loops.

## Prover path

The prover:

    validates C = <a, G> + rB
    builds a_ext = (a, r, 0, ..., 0)
    builds b_ext = (eq(z, ·), 0, ..., 0)
    builds G_ext = (G, B, padding generators)
    uses the existing IPA prover opening loop on the extended relation

## Verifier path

The verifier:

    reconstructs the extended generator basis
    constructs the extended point (z, 0)
    constructs the extended public statement
    uses the real IPA verifier opening loop

The verifier does not know the blinding scalar.

## Production boundary

This branch removes the previous fail-closed limitation for blinded commitments by adding a real extended-relation path.

It does not weaken `verify_ipa_opening`; the unblinded verifier remains strict.

## Security rule

Padding generators and the extended basis blinding generator must be generated independently with domain separation.

Do not reuse generator labels.
