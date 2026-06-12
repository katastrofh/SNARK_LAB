#!/usr/bin/env bash
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

echo "[1/16] cargo fmt"
cargo fmt --all -- --check

echo "[2/16] cargo clippy"
cargo clippy --locked --workspace --all-targets -- -D warnings

echo "[3/16] cargo test"
cargo test --locked --workspace

echo "[4/16] fuzz target build"
scripts/check-fuzz-targets.sh

echo "[5/16] public test vectors"
scripts/check-test-vectors.sh

echo "[6/16] SRS ceremony spec"
scripts/check-srs-ceremony-spec.sh

echo "[7/16] deployment evidence pack"
scripts/check-deployment-evidence-pack.sh

echo "[8/16] audit readiness packet"
scripts/check-audit-readiness-packet.sh

echo "[9/16] release-candidate evidence"
scripts/check-release-candidate-evidence.sh

echo "[10/16] release checklist"
scripts/check-release-checklist.sh

echo "[11/16] GitHub release artifact tooling"
scripts/check-github-release-artifacts.sh

echo "[12/16] long fuzz campaign evidence"
scripts/check-long-fuzz-campaign-evidence.sh

echo "[13/16] visualizer build"
(
  cd web/visualizer
  npm ci
  npm run build
)

echo "[14/16] reject unsafe Rust"
if grep -RIn --include='*.rs' -E '\bunsafe\s*(\{|fn|impl|trait|extern)' crates fuzz; then
  echo "unsafe Rust found" >&2
  exit 1
fi

echo "[15/16] reject Number.isNaN in visualizer"
if grep -RIn 'Number\.isNaN' web/visualizer/src; then
  echo "Number.isNaN found; use the repo's explicit finite-number checks instead" >&2
  exit 1
fi

echo "[16/16] git working tree summary"
git status --short

echo "production readiness checks passed"
