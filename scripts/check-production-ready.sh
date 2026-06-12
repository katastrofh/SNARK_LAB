#!/usr/bin/env bash
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

echo "[1/13] cargo fmt"
cargo fmt --all -- --check

echo "[2/13] cargo clippy"
cargo clippy --locked --workspace --all-targets -- -D warnings

echo "[3/13] cargo test"
cargo test --locked --workspace

echo "[4/13] fuzz target build"
scripts/check-fuzz-targets.sh

echo "[5/13] public test vectors"
scripts/check-test-vectors.sh

echo "[6/13] SRS ceremony spec"
scripts/check-srs-ceremony-spec.sh

echo "[7/13] deployment evidence pack"
scripts/check-deployment-evidence-pack.sh

echo "[8/13] audit readiness packet"
scripts/check-audit-readiness-packet.sh

echo "[9/13] release-candidate evidence"
scripts/check-release-candidate-evidence.sh

echo "[10/13] visualizer build"
(
  cd web/visualizer
  npm ci
  npm run build
)

echo "[11/13] reject unsafe Rust"
if grep -RIn --include='*.rs' -E '\bunsafe\s*(\{|fn|impl|trait|extern)' crates fuzz; then
  echo "unsafe Rust found" >&2
  exit 1
fi

echo "[12/13] reject Number.isNaN in visualizer"
if grep -RIn 'Number\.isNaN' web/visualizer/src; then
  echo "Number.isNaN found; use the repo's explicit finite-number checks instead" >&2
  exit 1
fi

echo "[13/13] git working tree summary"
git status --short

echo "production readiness checks passed"
