#!/usr/bin/env bash
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

echo "[1/28] cargo fmt"
cargo fmt --all -- --check

echo "[2/28] cargo clippy"
cargo clippy --locked --workspace --all-targets -- -D warnings

echo "[3/28] cargo test"
cargo test --locked --workspace

echo "[4/28] fuzz target build"
scripts/check-fuzz-targets.sh

echo "[5/28] public test vectors"
scripts/check-test-vectors.sh

echo "[6/28] SRS ceremony spec"
scripts/check-srs-ceremony-spec.sh

echo "[7/28] production SRS placeholder policy"
scripts/check-production-srs-placeholder-policy.sh

echo "[8/28] deployment evidence pack"
scripts/check-deployment-evidence-pack.sh

echo "[9/28] audit readiness packet"
scripts/check-audit-readiness-packet.sh

echo "[10/28] release-candidate evidence"
scripts/check-release-candidate-evidence.sh

echo "[11/28] release checklist"
scripts/check-release-checklist.sh

echo "[12/28] GitHub release artifact tooling"
scripts/check-github-release-artifacts.sh

echo "[13/28] GitHub release page finalization"
scripts/check-github-release-page-finalization.sh

echo "[14/28] manual GitHub release publication evidence"
scripts/check-manual-github-release-publication-evidence.sh

echo "[15/28] rc2 current-main release candidate"
scripts/check-rc2-current-main-release-candidate.sh

echo "[16/28] GitHub release rc2 publication evidence"
scripts/check-github-release-rc2-publication.sh

echo "[17/28] long fuzz campaign evidence"
scripts/check-long-fuzz-campaign-evidence.sh

echo "[18/28] fuzz nightly runner policy"
scripts/check-fuzz-nightly-runner.sh

echo "[19/28] nightly fuzz smoke evidence"
scripts/check-nightly-fuzz-smoke-evidence.sh

echo "[20/28] all fuzz targets smoke evidence"
scripts/check-all-fuzz-targets-smoke-evidence.sh

echo "[21/28] fuzz crash regression suite"
scripts/check-fuzz-crash-regressions.sh

echo "[22/28] production deployment guide"
scripts/check-production-deployment-guide.sh

echo "[23/28] README star polish"
scripts/check-readme-star-polish.sh
echo "[24/28] visualizer screenshot assets"
scripts/check-visualizer-screenshot-assets.sh
echo "[25/28] visualizer build"
(
  cd web/visualizer
  npm ci
  npm run build
)

echo "[26/28] reject unsafe Rust"
if grep -RIn --include='*.rs' -E '\bunsafe\s*(\{|fn|impl|trait|extern)' crates fuzz; then
  echo "unsafe Rust found" >&2
  exit 1
fi

echo "[27/28] reject Number.isNaN in visualizer"
if grep -RIn 'Number\.isNaN' web/visualizer/src; then
  echo "Number.isNaN found; use the repo's explicit finite-number checks instead" >&2
  exit 1
fi

echo "[28/28] git working tree summary"
git status --short

echo "production readiness checks passed"
