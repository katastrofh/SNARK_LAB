# Visualizer Polish and Demo

This branch makes the visualizer more suitable for GitHub Pages, demos, screenshots, and repository review.

## Added

- direct tab links through the `tab` query parameter
- visualizer-local README
- screenshot and GIF capture checklist
- responsive polish for smaller screens
- print/screenshot-friendly CSS boundaries

## Recommended demo path

Open the IPA tab directly:

    ?tab=ipa

Then show:

1. Statement and evaluation basis
2. Rust demo vector metadata
3. Round-by-round IPA folding
4. Final folded scalars
5. Explicit security boundary

## Screenshot checklist

Use a browser width around 1440px.

Capture:

- the top of the IPA page
- the round-by-round folding panel
- the final Rust-core status cards
- the security-boundary banner

## GIF checklist

A short GIF should show:

1. opening the IPA tab
2. selecting reduction round 1
3. selecting reduction round 2
4. scrolling to the implemented/boundary cards

Keep the GIF under 20 seconds.

## Boundary

The visualizer remains educational.

It must not be described as a cryptographic audit, production deployment proof, or production SRS evidence.
