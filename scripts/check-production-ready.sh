#!/usr/bin/env bash
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

echo "[1/21] cargo fmt"
cargo fmt --all -- --check

echo "[2/21] cargo clippy"
cargo clippy --locked --workspace --all-targets -- -D warnings

echo "[3/21] cargo test"
cargo test --locked --workspace

echo "[4/21] fuzz target build"
scripts/check-fuzz-targets.sh

echo "[5/21] public test vectors"
scripts/check-test-vectors.sh

echo "[6/21] SRS ceremony spec"
scripts/check-srs-ceremony-spec.sh

echo "[7/21] production SRS placeholder policy"
scripts/check-production-srs-placeholder-policy.sh

echo "[8/21] deployment evidence pack"
scripts/check-deployment-evidence-pack.sh

echo "[9/21] audit readiness packet"
scripts/check-audit-readiness-packet.sh

echo "[10/21] release-candidate evidence"
scripts/check-release-candidate-evidence.sh

echo "[11/21] release checklist"
scripts/check-release-checklist.sh

echo "[12/21] GitHub release artifact tooling"
scripts/check-github-release-artifacts.sh

echo "[13/21] long fuzz campaign evidence"
scripts/check-long-fuzz-campaign-evidence.sh

echo "[14/21] fuzz nightly runner policy"
scripts/check-fuzz-nightly-runner.sh

echo "[15/21] nightly fuzz smoke evidence"
scripts/check-nightly-fuzz-smoke-evidence.sh

echo "[16/21] all fuzz targets smoke evidence"
scripts/check-all-fuzz-targets-smoke-evidence.sh

echo "[17/21] production deployment guide"
scripts/check-production-deployment-guide.sh

echo "[18/21] visualizer build"
(
  cd web/visualizer
  npm ci
  npm run build
)

echo "[19/21] reject unsafe Rust"
if grep -RIn --include='*.rs' -E '\bunsafe\s*(\{|fn|impl|trait|extern)' crates fuzz; then
  echo "unsafe Rust found" >&2
  exit 1
fi

echo "[20/21] reject Number.isNaN in visualizer"
if grep -RIn 'Number\.isNaN' web/visualizer/src; then
  echo "Number.isNaN found; use the repo's explicit finite-number checks instead" >&2
  exit 1
fi

echo "[21/21] git working tree summary"
git status --short

echo "production readiness checks passed"
