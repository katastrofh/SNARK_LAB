#!/usr/bin/env bash
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

echo "[1/30] cargo fmt"
cargo fmt --all -- --check

echo "[2/30] cargo clippy"
cargo clippy --locked --workspace --all-targets -- -D warnings

echo "[3/30] cargo test"
cargo test --locked --workspace

echo "[4/30] fuzz target build"
scripts/check-fuzz-targets.sh

echo "[5/30] public test vectors"
scripts/check-test-vectors.sh

echo "[6/30] SRS ceremony spec"
scripts/check-srs-ceremony-spec.sh

echo "[7/30] production SRS placeholder policy"
scripts/check-production-srs-placeholder-policy.sh

echo "[8/30] deployment evidence pack"
scripts/check-deployment-evidence-pack.sh

echo "[9/30] audit readiness packet"
scripts/check-audit-readiness-packet.sh

echo "[10/30] release-candidate evidence"
scripts/check-release-candidate-evidence.sh

echo "[11/30] release checklist"
scripts/check-release-checklist.sh

echo "[12/30] GitHub release artifact tooling"
scripts/check-github-release-artifacts.sh

echo "[13/30] GitHub release page finalization"
scripts/check-github-release-page-finalization.sh

echo "[14/30] manual GitHub release publication evidence"
scripts/check-manual-github-release-publication-evidence.sh

echo "[15/30] rc2 current-main release candidate"
scripts/check-rc2-current-main-release-candidate.sh

echo "[16/30] GitHub release rc2 publication evidence"
scripts/check-github-release-rc2-publication.sh

echo "[17/30] long fuzz campaign evidence"
scripts/check-long-fuzz-campaign-evidence.sh

echo "[18/30] fuzz nightly runner policy"
scripts/check-fuzz-nightly-runner.sh

echo "[19/30] nightly fuzz smoke evidence"
scripts/check-nightly-fuzz-smoke-evidence.sh

echo "[20/30] all fuzz targets smoke evidence"
scripts/check-all-fuzz-targets-smoke-evidence.sh

echo "[21/30] fuzz crash regression suite"
scripts/check-fuzz-crash-regressions.sh

echo "[22/30] production deployment guide"
scripts/check-production-deployment-guide.sh

echo "[23/30] README star polish"
scripts/check-readme-star-polish.sh

echo "[24/30] final project positioning and roadmap"
scripts/check-final-project-positioning-and-roadmap.sh

echo "[25/30] final repository health report"
scripts/check-final-repo-health-report.sh
echo "[26/30] visualizer screenshot assets"
scripts/check-visualizer-screenshot-assets.sh
echo "[27/30] visualizer build"
(
  cd web/visualizer
  npm ci
  npm run build
)

echo "[28/30] reject unsafe Rust"
if grep -RIn --include='*.rs' -E '\bunsafe\s*(\{|fn|impl|trait|extern)' crates fuzz; then
  echo "unsafe Rust found" >&2
  exit 1
fi

echo "[29/30] reject Number.isNaN in visualizer"
if grep -RIn 'Number\.isNaN' web/visualizer/src; then
  echo "Number.isNaN found; use the repo's explicit finite-number checks instead" >&2
  exit 1
fi

echo "[30/30] git working tree summary"
git status --short

echo "production readiness checks passed"
