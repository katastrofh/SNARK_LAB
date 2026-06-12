# IPA Verifier Opening Loop

This document defines the verifier-side IPA opening loop for the currently supported unblinded relation.

## Verifier recurrence

The verifier starts from the public relation commitment:

    P = C + <eq(z, ·), H> + vU

For each round with transcript challenge `x`, it updates:

    P' = P + x^2 L + x^{-2} R

It also folds the generator basis consistently with the prover.

## Final check

At the end, the verifier checks the folded final relation:

    P_final = a_final G_final + b_final H_final + (a_final b_final) U

The verifier does not separately require `a_final * b_final = v`, because the recursive update accumulates the cross terms into the `U` coefficient.

## What this adds

The oracle crate now exposes:

    verify_ipa_opening
    IpaVerifierOpeningError

## Production boundary

This is a real verifier-side recursive check for the currently supported unblinded relation.

It intentionally rejects nonzero-blinded commitments until the blinding-opening extension is implemented.

## Security rule

Do not weaken this verifier to accept unsupported hiding commitments.

Unsupported commitments must fail closed.
