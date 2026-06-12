#!/usr/bin/env bash
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

echo "[1/9] cargo fmt"
cargo fmt --all -- --check

echo "[2/9] cargo clippy"
cargo clippy --locked --workspace --all-targets -- -D warnings

echo "[3/9] cargo test"
cargo test --locked --workspace

echo "[4/9] fuzz target build"
cargo check --locked --manifest-path fuzz/Cargo.toml --bins

echo "[5/9] public test vectors"
scripts/check-test-vectors.sh

echo "[6/9] visualizer build"
(
  cd web/visualizer
  npm ci
  npm run build
)

echo "[7/9] reject unsafe Rust"
if grep -RIn --include='*.rs' -E '\bunsafe\s*(\{|fn|impl|trait|extern)' crates fuzz; then
  echo "unsafe Rust found" >&2
  exit 1
fi

echo "[8/9] reject Number.isNaN in visualizer"
if grep -RIn 'Number\.isNaN' web/visualizer/src; then
  echo "Number.isNaN found; use the repo's explicit finite-number checks instead" >&2
  exit 1
fi

echo "[9/9] git working tree summary"
git status --short

echo "production readiness checks passed"
