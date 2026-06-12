#!/usr/bin/env bash
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

echo "[1/17] cargo fmt"
cargo fmt --all -- --check

echo "[2/17] cargo clippy"
cargo clippy --locked --workspace --all-targets -- -D warnings

echo "[3/17] cargo test"
cargo test --locked --workspace

echo "[4/17] fuzz target build"
scripts/check-fuzz-targets.sh

echo "[5/17] public test vectors"
scripts/check-test-vectors.sh

echo "[6/17] SRS ceremony spec"
scripts/check-srs-ceremony-spec.sh

echo "[7/17] deployment evidence pack"
scripts/check-deployment-evidence-pack.sh

echo "[8/17] audit readiness packet"
scripts/check-audit-readiness-packet.sh

echo "[9/17] release-candidate evidence"
scripts/check-release-candidate-evidence.sh

echo "[10/17] release checklist"
scripts/check-release-checklist.sh

echo "[11/17] GitHub release artifact tooling"
scripts/check-github-release-artifacts.sh

echo "[12/17] long fuzz campaign evidence"
scripts/check-long-fuzz-campaign-evidence.sh

echo "[13/17] production deployment guide"
scripts/check-production-deployment-guide.sh

echo "[14/17] visualizer build"
(
  cd web/visualizer
  npm ci
  npm run build
)

echo "[15/17] reject unsafe Rust"
if grep -RIn --include='*.rs' -E '\bunsafe\s*(\{|fn|impl|trait|extern)' crates fuzz; then
  echo "unsafe Rust found" >&2
  exit 1
fi

echo "[16/17] reject Number.isNaN in visualizer"
if grep -RIn 'Number\.isNaN' web/visualizer/src; then
  echo "Number.isNaN found; use the repo's explicit finite-number checks instead" >&2
  exit 1
fi

echo "[17/17] git working tree summary"
git status --short

echo "production readiness checks passed"
