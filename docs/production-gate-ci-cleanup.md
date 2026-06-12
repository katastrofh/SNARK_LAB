# Production Gate and CI Cleanup

This branch fixes the local production gate and tracks the CI/audit files.

## Fixes

The local production gate now runs:

    cargo fmt --all -- --check
    cargo clippy --locked --workspace --all-targets -- -D warnings
    cargo test --locked --workspace
    cargo check --locked --manifest-path fuzz/Cargo.toml --bins
    npm ci
    npm run build
    unsafe Rust rejection
    visualizer NaN-footgun rejection
    git working-tree summary

## CI

GitHub Actions now checks:

    Linux Rust build
    macOS Rust build
    visualizer build
    fuzz target compilation
    RustSec audit
    npm high-severity audit

## Dependabot

Dependabot is configured for:

    Cargo
    npm
    GitHub Actions

## Production boundary

This branch changes repository hardening infrastructure only.

It does not change protocol logic.
