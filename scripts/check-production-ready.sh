#!/usr/bin/env bash
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

echo "[1/18] cargo fmt"
cargo fmt --all -- --check

echo "[2/18] cargo clippy"
cargo clippy --locked --workspace --all-targets -- -D warnings

echo "[3/18] cargo test"
cargo test --locked --workspace

echo "[4/18] fuzz target build"
scripts/check-fuzz-targets.sh

echo "[5/18] public test vectors"
scripts/check-test-vectors.sh

echo "[6/18] SRS ceremony spec"
scripts/check-srs-ceremony-spec.sh

echo "[7/18] production SRS placeholder policy"
scripts/check-production-srs-placeholder-policy.sh

echo "[8/18] deployment evidence pack"
scripts/check-deployment-evidence-pack.sh

echo "[9/18] audit readiness packet"
scripts/check-audit-readiness-packet.sh

echo "[10/18] release-candidate evidence"
scripts/check-release-candidate-evidence.sh

echo "[11/18] release checklist"
scripts/check-release-checklist.sh

echo "[12/18] GitHub release artifact tooling"
scripts/check-github-release-artifacts.sh

echo "[13/18] long fuzz campaign evidence"
scripts/check-long-fuzz-campaign-evidence.sh

echo "[14/18] production deployment guide"
scripts/check-production-deployment-guide.sh

echo "[15/18] visualizer build"
(
  cd web/visualizer
  npm ci
  npm run build
)

echo "[16/18] reject unsafe Rust"
if grep -RIn --include='*.rs' -E '\bunsafe\s*(\{|fn|impl|trait|extern)' crates fuzz; then
  echo "unsafe Rust found" >&2
  exit 1
fi

echo "[17/18] reject Number.isNaN in visualizer"
if grep -RIn 'Number\.isNaN' web/visualizer/src; then
  echo "Number.isNaN found; use the repo's explicit finite-number checks instead" >&2
  exit 1
fi

echo "[18/18] git working tree summary"
git status --short

echo "production readiness checks passed"
