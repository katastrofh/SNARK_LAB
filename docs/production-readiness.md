# Production Readiness

This repository is moving from educational protocol implementations toward production-shaped zkSNARK components.

## Current production-shaped pieces

The following boundaries are now explicit:

- Fiat-Shamir transcripts are domain separated.
- Sumcheck has an optimized multilinear-table path.
- Sumcheck has a general bounded-degree API.
- Sumcheck has an oracle-backed verification path.
- The oracle abstraction separates protocol logic from commitment/opening logic.
- The transparent oracle backend is isolated as a replaceable backend.
- CI and local production checks run Rust formatting, clippy, tests, and visualizer builds.

## Current non-production pieces

The repository is not yet production cryptography.

Known remaining gaps:

- `TransparentOracle` is not succinct.
- `TransparentOracle` commitments contain the full evaluation table.
- There is no real KZG, IPA, or FRI backend yet.
- There is no audited serialization format for network/proof bytes yet.
- There is no formal security proof document yet.
- The browser visualizer remains educational and must not be treated as a verifier.

## Production verifier target

The target production verifier should receive only:

    public statement
    oracle commitment
    transcript-bound Sumcheck proof
    authenticated final opening proof

It should not receive the full witness table.

## Backend roadmap

The transparent backend should eventually be replaced by one of:

- KZG polynomial commitment backend,
- IPA polynomial commitment backend,
- FRI/STARK-style commitment backend.

The current oracle trait exists so that this replacement does not require rewriting Sumcheck.

## Local production check

Run:

    scripts/check-production-ready.sh

This script enforces:

- rustfmt,
- clippy with warnings denied,
- full workspace tests,
- visualizer build,
- no unsafe Rust,
- no `Number.isNaN` in the visualizer.
