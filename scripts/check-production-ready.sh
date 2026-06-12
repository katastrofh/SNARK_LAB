#!/usr/bin/env bash
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

echo "[1/25] cargo fmt"
cargo fmt --all -- --check

echo "[2/25] cargo clippy"
cargo clippy --locked --workspace --all-targets -- -D warnings

echo "[3/25] cargo test"
cargo test --locked --workspace

echo "[4/25] fuzz target build"
scripts/check-fuzz-targets.sh

echo "[5/25] public test vectors"
scripts/check-test-vectors.sh

echo "[6/25] SRS ceremony spec"
scripts/check-srs-ceremony-spec.sh

echo "[7/25] production SRS placeholder policy"
scripts/check-production-srs-placeholder-policy.sh

echo "[8/25] deployment evidence pack"
scripts/check-deployment-evidence-pack.sh

echo "[9/25] audit readiness packet"
scripts/check-audit-readiness-packet.sh

echo "[10/25] release-candidate evidence"
scripts/check-release-candidate-evidence.sh

echo "[11/25] release checklist"
scripts/check-release-checklist.sh

echo "[12/25] GitHub release artifact tooling"
scripts/check-github-release-artifacts.sh

echo "[13/25] GitHub release page finalization"
scripts/check-github-release-page-finalization.sh

echo "[14/25] long fuzz campaign evidence"
scripts/check-long-fuzz-campaign-evidence.sh

echo "[15/25] fuzz nightly runner policy"
scripts/check-fuzz-nightly-runner.sh

echo "[16/25] nightly fuzz smoke evidence"
scripts/check-nightly-fuzz-smoke-evidence.sh

echo "[17/25] all fuzz targets smoke evidence"
scripts/check-all-fuzz-targets-smoke-evidence.sh

echo "[18/25] fuzz crash regression suite"
scripts/check-fuzz-crash-regressions.sh

echo "[19/25] production deployment guide"
scripts/check-production-deployment-guide.sh

echo "[20/25] README star polish"
scripts/check-readme-star-polish.sh
echo "[21/25] visualizer screenshot assets"
scripts/check-visualizer-screenshot-assets.sh
echo "[22/25] visualizer build"
(
  cd web/visualizer
  npm ci
  npm run build
)

echo "[23/25] reject unsafe Rust"
if grep -RIn --include='*.rs' -E '\bunsafe\s*(\{|fn|impl|trait|extern)' crates fuzz; then
  echo "unsafe Rust found" >&2
  exit 1
fi

echo "[24/25] reject Number.isNaN in visualizer"
if grep -RIn 'Number\.isNaN' web/visualizer/src; then
  echo "Number.isNaN found; use the repo's explicit finite-number checks instead" >&2
  exit 1
fi

echo "[25/25] git working tree summary"
git status --short

echo "production readiness checks passed"
