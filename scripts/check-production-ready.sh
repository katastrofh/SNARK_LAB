#!/usr/bin/env bash
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

echo "[1/8] cargo fmt"
cargo fmt --all -- --check

echo "[2/8] cargo clippy"
cargo clippy --locked --workspace --all-targets -- -D warnings

echo "[3/8] cargo test"
cargo test --locked --workspace

echo "[4/8] fuzz target build"
cargo check --locked --manifest-path fuzz/Cargo.toml --bins

echo "[5/8] visualizer build"
(
  cd web/visualizer
  npm ci
  npm run build
)

echo "[6/8] reject unsafe Rust"
if grep -RIn --include='*.rs' -E '\bunsafe\s*(\{|fn|impl|trait|extern)' crates fuzz; then
  echo "unsafe Rust found" >&2
  exit 1
fi

echo "[7/8] reject Number.isNaN in visualizer"
if grep -RIn 'Number\.isNaN' web/visualizer/src; then
  echo "Number.isNaN found; use the repo's explicit finite-number checks instead" >&2
  exit 1
fi

echo "[8/8] git working tree summary"
git status --short

echo "production readiness checks passed"
