#!/usr/bin/env bash
set -euo pipefail

echo "[1/7] cargo fmt"
cargo fmt --all -- --check

echo "[2/7] cargo clippy"
cargo clippy --workspace --all-targets -- -D warnings

echo "[3/7] cargo test"
cargo test --workspace

echo "[4/7] visualizer build"
npm --prefix web/visualizer run build

echo "[5/7] reject unsafe Rust"
if grep -RIn --include='*.rs' 'unsafe ' crates; then
  echo "unsafe Rust found; production hardening forbids this"
  exit 1
fi

echo "[6/7] reject Number.isNaN in visualizer"
if grep -RIn 'Number.isNaN' web/visualizer/src; then
  echo "Use Number.isSafeInteger / explicit validation instead"
  exit 1
fi

echo "[7/7] git working tree summary"
git status --short

echo "production readiness checks passed"
