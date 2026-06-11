# Oracle-backed Sumcheck

This note describes the commitment-shaped Sumcheck path.

## Current transparent verifier

The original optimized verifier receives the full multilinear table and checks the final value by directly evaluating the table.

That is useful for testing and education, but it is not succinct.

## Oracle-backed verifier

The oracle-backed verifier receives:

    oracle commitment
    claimed sum
    Sumcheck proof
    final opening proof

The verifier binds the commitment into the Fiat-Shamir transcript, derives the same round challenges, checks every Sumcheck round, and verifies the final oracle opening.

## Current backend

The first backend is `TransparentOracle`.

It is intentionally non-succinct because its commitment contains the full evaluation table.

The purpose is to make the protocol boundary correct now, so a later KZG, IPA, or FRI backend can replace the transparent opening logic without rewriting Sumcheck.

## Security checks

The tests cover:

    valid transparent opening
    tampered final opening
    changed commitment
    tampered round polynomial
    wrong round count

## Production roadmap

The next backend should replace `TransparentOracle` with a succinct polynomial commitment scheme.

The verifier should eventually check:

    commitment
    transcript-bound Sumcheck proof
    authenticated opening at final point

without receiving the full witness table.
