# Local Production Gates

This repository includes a local pre-push hook.

The hook runs:

    scripts/check-production-ready.sh

before allowing `git push`.

## Install

Run:

    scripts/install-git-hooks.sh

This sets:

    git config core.hooksPath .githooks

## What this blocks

The hook blocks pushes with:

    rustfmt failures
    clippy warnings
    failing Rust tests
    failing visualizer builds
    unsafe Rust
    banned browser validation patterns

## Bypass policy

Do not bypass this hook during normal development.

If an emergency bypass is required, use Git's explicit bypass flag and document why:

    git push --no-verify
