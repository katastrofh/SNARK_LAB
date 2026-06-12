#!/usr/bin/env bash
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

echo "[1/10] cargo fmt"
cargo fmt --all -- --check

echo "[2/10] cargo clippy"
cargo clippy --locked --workspace --all-targets -- -D warnings

echo "[3/10] cargo test"
cargo test --locked --workspace

echo "[4/10] fuzz target build"
scripts/check-fuzz-targets.sh

echo "[5/10] public test vectors"
scripts/check-test-vectors.sh

echo "[6/10] SRS ceremony spec"
scripts/check-srs-ceremony-spec.sh

echo "[7/10] visualizer build"
(
  cd web/visualizer
  npm ci
  npm run build
)

echo "[8/10] reject unsafe Rust"
if grep -RIn --include='*.rs' -E '\bunsafe\s*(\{|fn|impl|trait|extern)' crates fuzz; then
  echo "unsafe Rust found" >&2
  exit 1
fi

echo "[9/10] reject Number.isNaN in visualizer"
if grep -RIn 'Number\.isNaN' web/visualizer/src; then
  echo "Number.isNaN found; use the repo's explicit finite-number checks instead" >&2
  exit 1
fi

echo "[10/10] git working tree summary"
git status --short

echo "production readiness checks passed"
