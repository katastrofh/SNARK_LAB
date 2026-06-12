#!/usr/bin/env bash
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

echo "[1/19] cargo fmt"
cargo fmt --all -- --check

echo "[2/19] cargo clippy"
cargo clippy --locked --workspace --all-targets -- -D warnings

echo "[3/19] cargo test"
cargo test --locked --workspace

echo "[4/19] fuzz target build"
scripts/check-fuzz-targets.sh

echo "[5/19] public test vectors"
scripts/check-test-vectors.sh

echo "[6/19] SRS ceremony spec"
scripts/check-srs-ceremony-spec.sh

echo "[7/19] production SRS placeholder policy"
scripts/check-production-srs-placeholder-policy.sh

echo "[8/19] deployment evidence pack"
scripts/check-deployment-evidence-pack.sh

echo "[9/19] audit readiness packet"
scripts/check-audit-readiness-packet.sh

echo "[10/19] release-candidate evidence"
scripts/check-release-candidate-evidence.sh

echo "[11/19] release checklist"
scripts/check-release-checklist.sh

echo "[12/19] GitHub release artifact tooling"
scripts/check-github-release-artifacts.sh

echo "[13/19] long fuzz campaign evidence"
scripts/check-long-fuzz-campaign-evidence.sh

echo "[14/19] fuzz nightly runner policy"
scripts/check-fuzz-nightly-runner.sh

echo "[15/19] production deployment guide"
scripts/check-production-deployment-guide.sh

echo "[16/19] visualizer build"
(
  cd web/visualizer
  npm ci
  npm run build
)

echo "[17/19] reject unsafe Rust"
if grep -RIn --include='*.rs' -E '\bunsafe\s*(\{|fn|impl|trait|extern)' crates fuzz; then
  echo "unsafe Rust found" >&2
  exit 1
fi

echo "[18/19] reject Number.isNaN in visualizer"
if grep -RIn 'Number\.isNaN' web/visualizer/src; then
  echo "Number.isNaN found; use the repo's explicit finite-number checks instead" >&2
  exit 1
fi

echo "[19/19] git working tree summary"
git status --short

echo "production readiness checks passed"
