#!/usr/bin/env bash
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

echo "[1/23] cargo fmt"
cargo fmt --all -- --check

echo "[2/23] cargo clippy"
cargo clippy --locked --workspace --all-targets -- -D warnings

echo "[3/23] cargo test"
cargo test --locked --workspace

echo "[4/23] fuzz target build"
scripts/check-fuzz-targets.sh

echo "[5/23] public test vectors"
scripts/check-test-vectors.sh

echo "[6/23] SRS ceremony spec"
scripts/check-srs-ceremony-spec.sh

echo "[7/23] production SRS placeholder policy"
scripts/check-production-srs-placeholder-policy.sh

echo "[8/23] deployment evidence pack"
scripts/check-deployment-evidence-pack.sh

echo "[9/23] audit readiness packet"
scripts/check-audit-readiness-packet.sh

echo "[10/23] release-candidate evidence"
scripts/check-release-candidate-evidence.sh

echo "[11/23] release checklist"
scripts/check-release-checklist.sh

echo "[12/23] GitHub release artifact tooling"
scripts/check-github-release-artifacts.sh

echo "[13/23] long fuzz campaign evidence"
scripts/check-long-fuzz-campaign-evidence.sh

echo "[14/23] fuzz nightly runner policy"
scripts/check-fuzz-nightly-runner.sh

echo "[15/23] nightly fuzz smoke evidence"
scripts/check-nightly-fuzz-smoke-evidence.sh

echo "[16/23] all fuzz targets smoke evidence"
scripts/check-all-fuzz-targets-smoke-evidence.sh

echo "[17/23] fuzz crash regression suite"
scripts/check-fuzz-crash-regressions.sh

echo "[18/23] production deployment guide"
scripts/check-production-deployment-guide.sh

echo "[19/23] README star polish"
scripts/check-readme-star-polish.sh
echo "[20/23] visualizer build"
(
  cd web/visualizer
  npm ci
  npm run build
)

echo "[21/23] reject unsafe Rust"
if grep -RIn --include='*.rs' -E '\bunsafe\s*(\{|fn|impl|trait|extern)' crates fuzz; then
  echo "unsafe Rust found" >&2
  exit 1
fi

echo "[22/23] reject Number.isNaN in visualizer"
if grep -RIn 'Number\.isNaN' web/visualizer/src; then
  echo "Number.isNaN found; use the repo's explicit finite-number checks instead" >&2
  exit 1
fi

echo "[23/23] git working tree summary"
git status --short

echo "production readiness checks passed"
