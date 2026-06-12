#!/usr/bin/env bash
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

echo "[1/26] cargo fmt"
cargo fmt --all -- --check

echo "[2/26] cargo clippy"
cargo clippy --locked --workspace --all-targets -- -D warnings

echo "[3/26] cargo test"
cargo test --locked --workspace

echo "[4/26] fuzz target build"
scripts/check-fuzz-targets.sh

echo "[5/26] public test vectors"
scripts/check-test-vectors.sh

echo "[6/26] SRS ceremony spec"
scripts/check-srs-ceremony-spec.sh

echo "[7/26] production SRS placeholder policy"
scripts/check-production-srs-placeholder-policy.sh

echo "[8/26] deployment evidence pack"
scripts/check-deployment-evidence-pack.sh

echo "[9/26] audit readiness packet"
scripts/check-audit-readiness-packet.sh

echo "[10/26] release-candidate evidence"
scripts/check-release-candidate-evidence.sh

echo "[11/26] release checklist"
scripts/check-release-checklist.sh

echo "[12/26] GitHub release artifact tooling"
scripts/check-github-release-artifacts.sh

echo "[13/26] GitHub release page finalization"
scripts/check-github-release-page-finalization.sh

echo "[14/26] manual GitHub release publication evidence"
scripts/check-manual-github-release-publication-evidence.sh

echo "[15/26] long fuzz campaign evidence"
scripts/check-long-fuzz-campaign-evidence.sh

echo "[16/26] fuzz nightly runner policy"
scripts/check-fuzz-nightly-runner.sh

echo "[17/26] nightly fuzz smoke evidence"
scripts/check-nightly-fuzz-smoke-evidence.sh

echo "[18/26] all fuzz targets smoke evidence"
scripts/check-all-fuzz-targets-smoke-evidence.sh

echo "[19/26] fuzz crash regression suite"
scripts/check-fuzz-crash-regressions.sh

echo "[20/26] production deployment guide"
scripts/check-production-deployment-guide.sh

echo "[21/26] README star polish"
scripts/check-readme-star-polish.sh
echo "[22/26] visualizer screenshot assets"
scripts/check-visualizer-screenshot-assets.sh
echo "[23/26] visualizer build"
(
  cd web/visualizer
  npm ci
  npm run build
)

echo "[24/26] reject unsafe Rust"
if grep -RIn --include='*.rs' -E '\bunsafe\s*(\{|fn|impl|trait|extern)' crates fuzz; then
  echo "unsafe Rust found" >&2
  exit 1
fi

echo "[25/26] reject Number.isNaN in visualizer"
if grep -RIn 'Number\.isNaN' web/visualizer/src; then
  echo "Number.isNaN found; use the repo's explicit finite-number checks instead" >&2
  exit 1
fi

echo "[26/26] git working tree summary"
git status --short

echo "production readiness checks passed"
