# SNARK_LAB Visualizer

The visualizer is an educational browser interface for the protocol lab.

It runs locally in the browser and uses small-field arithmetic for inspection.

## Local development

Run:

    cd web/visualizer
    npm ci
    npm run dev

## Production build

Run:

    cd web/visualizer
    npm ci
    npm run build

The repository production gate also builds the visualizer.

## Demo URLs

The visualizer supports direct tab links:

    /?tab=sumcheck
    /?tab=zerocheck
    /?tab=permcheck
    /?tab=scribe
    /?tab=ipa

The IPA tab is the recommended demo entry point for the current repository state.

## Security boundary

The browser visualizer is educational.

The Rust implementation uses BLS12-381, Merlin Fiat-Shamir, typed IPA openings, SRS validation, negative tests, public test vectors, and production-gate checks.

The browser visualizer is not an audit and is not production deployment evidence.
