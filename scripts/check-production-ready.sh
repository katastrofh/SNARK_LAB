#!/usr/bin/env bash
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

echo "[1/36] cargo fmt"
cargo fmt --all -- --check

echo "[2/36] cargo clippy"
cargo clippy --locked --workspace --all-targets -- -D warnings

echo "[3/36] cargo test"
cargo test --locked --workspace

echo "[4/36] GitHub Actions dependency fetch retry policy"
scripts/check-github-actions-dependency-fetch-retry.sh

echo "[5/36] GitHub Actions cargo audit compatibility"
scripts/check-github-actions-cargo-audit.sh

echo "[6/36] fuzz target build"
scripts/check-fuzz-targets.sh

echo "[7/36] public test vectors"
scripts/check-test-vectors.sh

echo "[8/36] SRS ceremony spec"
scripts/check-srs-ceremony-spec.sh

echo "[9/36] production SRS placeholder policy"
scripts/check-production-srs-placeholder-policy.sh

echo "[10/36] deployment evidence pack"
scripts/check-deployment-evidence-pack.sh

echo "[11/36] audit readiness packet"
scripts/check-audit-readiness-packet.sh

echo "[12/36] release-candidate evidence"
scripts/check-release-candidate-evidence.sh

echo "[13/36] release checklist"
scripts/check-release-checklist.sh

echo "[14/36] GitHub release artifact tooling"
scripts/check-github-release-artifacts.sh

echo "[15/36] GitHub release page finalization"
scripts/check-github-release-page-finalization.sh

echo "[16/36] manual GitHub release publication evidence"
scripts/check-manual-github-release-publication-evidence.sh

echo "[17/36] rc2 current-main release candidate"
scripts/check-rc2-current-main-release-candidate.sh

echo "[18/36] GitHub release rc2 publication evidence"
scripts/check-github-release-rc2-publication.sh

echo "[19/36] long fuzz campaign evidence"
scripts/check-long-fuzz-campaign-evidence.sh

echo "[20/36] fuzz nightly runner policy"
scripts/check-fuzz-nightly-runner.sh

echo "[21/36] nightly fuzz smoke evidence"
scripts/check-nightly-fuzz-smoke-evidence.sh

echo "[22/36] all fuzz targets smoke evidence"
scripts/check-all-fuzz-targets-smoke-evidence.sh

echo "[23/36] fuzz crash regression suite"
scripts/check-fuzz-crash-regressions.sh

echo "[24/36] production deployment guide"
scripts/check-production-deployment-guide.sh

echo "[25/36] README star polish"
scripts/check-readme-star-polish.sh

echo "[26/36] final project positioning and roadmap"
scripts/check-final-project-positioning-and-roadmap.sh

echo "[27/36] final repository health report"
scripts/check-final-repo-health-report.sh

echo "[28/36] reviewer onboarding guide"
scripts/check-reviewer-onboarding-guide.sh

echo "[29/36] examples gallery"
scripts/check-examples-gallery.sh

echo "[30/36] paper-style technical overview"
scripts/check-paper-style-technical-overview.sh

echo "[31/36] final repo polish and freeze"
scripts/check-final-repo-polish-and-freeze.sh
echo "[32/36] visualizer screenshot assets"
scripts/check-visualizer-screenshot-assets.sh
echo "[33/36] visualizer build"
(
  cd web/visualizer
  npm ci
  npm run build
)

echo "[34/36] reject unsafe Rust"
if grep -RIn --include='*.rs' -E '\bunsafe\s*(\{|fn|impl|trait|extern)' crates fuzz; then
  echo "unsafe Rust found" >&2
  exit 1
fi

echo "[35/36] reject Number.isNaN in visualizer"
if grep -RIn 'Number\.isNaN' web/visualizer/src; then
  echo "Number.isNaN found; use the repo's explicit finite-number checks instead" >&2
  exit 1
fi

echo "[36/36] git working tree summary"
git status --short

echo "production readiness checks passed"
