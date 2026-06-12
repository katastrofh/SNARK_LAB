# CI Matrix and Audit

This branch adds GitHub-side hardening for the repository.

## CI matrix

The `CI Matrix` workflow runs on:

    ubuntu-latest
    macos-latest

It checks:

    cargo fmt --all -- --check
    cargo metadata --locked --workspace
    cargo clippy --locked --workspace --all-targets -- -D warnings
    cargo test --locked --workspace
    cargo check --locked --manifest-path fuzz/Cargo.toml --bins

## Visualizer build

The visualizer job runs:

    npm ci
    npm run build

using Node 22 and `package-lock.json`.

## Audit workflow

The `Security Audit` workflow runs:

    cargo audit --locked
    npm audit --audit-level=high

It runs on pushes, pull requests, a weekly schedule, and manual dispatch.

## Dependabot

Dependabot is configured for:

    Cargo dependencies
    npm dependencies
    GitHub Actions

## Production boundary

This branch does not change protocol logic.

It adds infrastructure so the repository is checked across platforms, with locked dependency resolution and advisory scanning.
