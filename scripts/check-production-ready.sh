#!/usr/bin/env bash
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

echo "[1/15] cargo fmt"
cargo fmt --all -- --check

echo "[2/15] cargo clippy"
cargo clippy --locked --workspace --all-targets -- -D warnings

echo "[3/15] cargo test"
cargo test --locked --workspace

echo "[4/15] fuzz target build"
scripts/check-fuzz-targets.sh

echo "[5/15] public test vectors"
scripts/check-test-vectors.sh

echo "[6/15] SRS ceremony spec"
scripts/check-srs-ceremony-spec.sh

echo "[7/15] deployment evidence pack"
scripts/check-deployment-evidence-pack.sh

echo "[8/15] audit readiness packet"
scripts/check-audit-readiness-packet.sh

echo "[9/15] release-candidate evidence"
scripts/check-release-candidate-evidence.sh

echo "[10/15] release checklist"
scripts/check-release-checklist.sh

echo "[11/15] GitHub release artifact tooling"
scripts/check-github-release-artifacts.sh

echo "[12/15] visualizer build"
(
  cd web/visualizer
  npm ci
  npm run build
)

echo "[13/15] reject unsafe Rust"
if grep -RIn --include='*.rs' -E '\bunsafe\s*(\{|fn|impl|trait|extern)' crates fuzz; then
  echo "unsafe Rust found" >&2
  exit 1
fi

echo "[14/15] reject Number.isNaN in visualizer"
if grep -RIn 'Number\.isNaN' web/visualizer/src; then
  echo "Number.isNaN found; use the repo's explicit finite-number checks instead" >&2
  exit 1
fi

echo "[15/15] git working tree summary"
git status --short

echo "production readiness checks passed"
