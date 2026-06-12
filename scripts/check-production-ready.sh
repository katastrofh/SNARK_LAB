#!/usr/bin/env bash
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

echo "[1/22] cargo fmt"
cargo fmt --all -- --check

echo "[2/22] cargo clippy"
cargo clippy --locked --workspace --all-targets -- -D warnings

echo "[3/22] cargo test"
cargo test --locked --workspace

echo "[4/22] fuzz target build"
scripts/check-fuzz-targets.sh

echo "[5/22] public test vectors"
scripts/check-test-vectors.sh

echo "[6/22] SRS ceremony spec"
scripts/check-srs-ceremony-spec.sh

echo "[7/22] production SRS placeholder policy"
scripts/check-production-srs-placeholder-policy.sh

echo "[8/22] deployment evidence pack"
scripts/check-deployment-evidence-pack.sh

echo "[9/22] audit readiness packet"
scripts/check-audit-readiness-packet.sh

echo "[10/22] release-candidate evidence"
scripts/check-release-candidate-evidence.sh

echo "[11/22] release checklist"
scripts/check-release-checklist.sh

echo "[12/22] GitHub release artifact tooling"
scripts/check-github-release-artifacts.sh

echo "[13/22] long fuzz campaign evidence"
scripts/check-long-fuzz-campaign-evidence.sh

echo "[14/22] fuzz nightly runner policy"
scripts/check-fuzz-nightly-runner.sh

echo "[15/22] nightly fuzz smoke evidence"
scripts/check-nightly-fuzz-smoke-evidence.sh

echo "[16/22] all fuzz targets smoke evidence"
scripts/check-all-fuzz-targets-smoke-evidence.sh

echo "[17/22] fuzz crash regression suite"
scripts/check-fuzz-crash-regressions.sh

echo "[18/22] production deployment guide"
scripts/check-production-deployment-guide.sh

echo "[19/22] visualizer build"
(
  cd web/visualizer
  npm ci
  npm run build
)

echo "[20/22] reject unsafe Rust"
if grep -RIn --include='*.rs' -E '\bunsafe\s*(\{|fn|impl|trait|extern)' crates fuzz; then
  echo "unsafe Rust found" >&2
  exit 1
fi

echo "[21/22] reject Number.isNaN in visualizer"
if grep -RIn 'Number\.isNaN' web/visualizer/src; then
  echo "Number.isNaN found; use the repo's explicit finite-number checks instead" >&2
  exit 1
fi

echo "[22/22] git working tree summary"
git status --short

echo "production readiness checks passed"
