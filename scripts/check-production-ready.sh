#!/usr/bin/env bash
set -euo pipefail

echo "[1/8] cargo fmt"
cargo fmt --all -- --check

echo "[2/8] cargo clippy"
cargo clippy --workspace --all-targets -- -D warnings

echo "[3/8] cargo test"
cargo test --workspace

echo "[4/8] fuzz target build"\ncargo check --locked --manifest-path fuzz/Cargo.toml --bins\n\necho "[5/8] visualizer build"
npm --prefix web/visualizer run build

echo "[6/8] reject unsafe Rust"
if grep -RIn --include='*.rs' 'unsafe ' crates; then
  echo "unsafe Rust found; production hardening forbids this"
  exit 1
fi

echo "[7/8] reject Number.isNaN in visualizer"
if grep -RIn 'Number.isNaN' web/visualizer/src; then
  echo "Use Number.isSafeInteger / explicit validation instead"
  exit 1
fi

echo "[8/8] git working tree summary"
git status --short

echo "production readiness checks passed"
