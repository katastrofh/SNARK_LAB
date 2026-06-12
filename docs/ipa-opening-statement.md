# IPA Opening Statement

This document defines the public statement consumed by a future IPA opening verifier.

## Statement

An IPA opening statement binds:

    commitment C
    opening point z
    claimed value v
    evaluation basis eq(z, ·)

The intended relation is:

    v = f(z) = <a, eq(z, ·)>

where `a` is the committed multilinear evaluation vector.

## What this adds

The oracle crate now exposes:

    IpaOpeningStatement
    IpaOpeningStatementError
    opening_statement_from_witness
    validate_ipa_opening_statement
    bind_ipa_opening_statement_context

## Production boundary

This branch still does not implement full IPA verification.

It defines and binds the verifier-side statement. Future IPA reduction-round verification will consume this statement.

## Security rule

`opening_statement_from_witness` is a prover-side or test-side helper.

A public verifier must not recompute the claim using the polynomial vector, because the verifier should not know the hidden witness.
