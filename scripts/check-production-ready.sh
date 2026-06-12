#!/usr/bin/env bash
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

echo "[1/12] cargo fmt"
cargo fmt --all -- --check

echo "[2/12] cargo clippy"
cargo clippy --locked --workspace --all-targets -- -D warnings

echo "[3/12] cargo test"
cargo test --locked --workspace

echo "[4/12] fuzz target build"
scripts/check-fuzz-targets.sh

echo "[5/12] public test vectors"
scripts/check-test-vectors.sh

echo "[6/12] SRS ceremony spec"
scripts/check-srs-ceremony-spec.sh

echo "[7/12] deployment evidence pack"
scripts/check-deployment-evidence-pack.sh

echo "[8/12] audit readiness packet"
scripts/check-audit-readiness-packet.sh

echo "[9/12] visualizer build"
(
  cd web/visualizer
  npm ci
  npm run build
)

echo "[10/12] reject unsafe Rust"
if grep -RIn --include='*.rs' -E '\bunsafe\s*(\{|fn|impl|trait|extern)' crates fuzz; then
  echo "unsafe Rust found" >&2
  exit 1
fi

echo "[11/12] reject Number.isNaN in visualizer"
if grep -RIn 'Number\.isNaN' web/visualizer/src; then
  echo "Number.isNaN found; use the repo's explicit finite-number checks instead" >&2
  exit 1
fi

echo "[12/12] git working tree summary"
git status --short

echo "production readiness checks passed"
