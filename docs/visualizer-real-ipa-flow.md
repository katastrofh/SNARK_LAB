# Visualizer Real IPA Flow

This branch upgrades the IPA PCS visualizer.

## Added

The IPA tab now shows:

- statement table
- opening point
- evaluation basis
- claimed value
- committed Rust demo vector metadata
- round-by-round IPA folding
- L/R commitment labels
- challenge and inverse challenge
- final folded scalars
- explicit security boundary

## Boundary

The browser model uses F97 for readability.

The Rust implementation uses BLS12-381, Merlin Fiat-Shamir, canonical encodings, negative proof fixtures, randomized roundtrip tests, SRS validation, and public test vectors.

The visualizer is not an audit and not production deployment evidence.

## Why this matters

The previous IPA tab described much of the opening system as future work.

The Rust implementation now has a real typed IPA path, so the visualizer should display the current protocol flow instead of an outdated roadmap.
