# Visualizer Screenshot Assets

This document records the GitHub-facing visualizer screenshot assets.

## Added screenshots

- docs/assets/visualizer/system-flow.png
- docs/assets/visualizer/ipa-flow.png
- docs/assets/visualizer/sumcheck-flow.png

## Capture script

Use:

    scripts/capture-visualizer-screenshots.sh

The script builds the visualizer, starts a local preview server, captures screenshots with a Chromium-compatible browser, and validates PNG output.

## Checker

Use:

    scripts/check-visualizer-screenshot-assets.sh

## Boundary

Screenshots are documentation assets.

They are not cryptographic evidence and do not replace Rust tests, fuzzing, audit, or production SRS evidence.
