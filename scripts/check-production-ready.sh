#!/usr/bin/env bash
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

echo "[1/20] cargo fmt"
cargo fmt --all -- --check

echo "[2/20] cargo clippy"
cargo clippy --locked --workspace --all-targets -- -D warnings

echo "[3/20] cargo test"
cargo test --locked --workspace

echo "[4/20] fuzz target build"
scripts/check-fuzz-targets.sh

echo "[5/20] public test vectors"
scripts/check-test-vectors.sh

echo "[6/20] SRS ceremony spec"
scripts/check-srs-ceremony-spec.sh

echo "[7/20] production SRS placeholder policy"
scripts/check-production-srs-placeholder-policy.sh

echo "[8/20] deployment evidence pack"
scripts/check-deployment-evidence-pack.sh

echo "[9/20] audit readiness packet"
scripts/check-audit-readiness-packet.sh

echo "[10/20] release-candidate evidence"
scripts/check-release-candidate-evidence.sh

echo "[11/20] release checklist"
scripts/check-release-checklist.sh

echo "[12/20] GitHub release artifact tooling"
scripts/check-github-release-artifacts.sh

echo "[13/20] long fuzz campaign evidence"
scripts/check-long-fuzz-campaign-evidence.sh

echo "[14/20] fuzz nightly runner policy"
scripts/check-fuzz-nightly-runner.sh

echo "[15/20] nightly fuzz smoke evidence"
scripts/check-nightly-fuzz-smoke-evidence.sh

echo "[16/20] production deployment guide"
scripts/check-production-deployment-guide.sh

echo "[17/20] visualizer build"
(
  cd web/visualizer
  npm ci
  npm run build
)

echo "[18/20] reject unsafe Rust"
if grep -RIn --include='*.rs' -E '\bunsafe\s*(\{|fn|impl|trait|extern)' crates fuzz; then
  echo "unsafe Rust found" >&2
  exit 1
fi

echo "[19/20] reject Number.isNaN in visualizer"
if grep -RIn 'Number\.isNaN' web/visualizer/src; then
  echo "Number.isNaN found; use the repo's explicit finite-number checks instead" >&2
  exit 1
fi

echo "[20/20] git working tree summary"
git status --short

echo "production readiness checks passed"
