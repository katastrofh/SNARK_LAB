#!/usr/bin/env bash
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

echo "[1/35] cargo fmt"
cargo fmt --all -- --check

echo "[2/35] cargo clippy"
cargo clippy --locked --workspace --all-targets -- -D warnings

echo "[3/35] cargo test"
cargo test --locked --workspace

echo "[4/35] GitHub Actions dependency fetch retry policy"
scripts/check-github-actions-dependency-fetch-retry.sh

echo "[5/35] fuzz target build"
scripts/check-fuzz-targets.sh

echo "[6/35] public test vectors"
scripts/check-test-vectors.sh

echo "[7/35] SRS ceremony spec"
scripts/check-srs-ceremony-spec.sh

echo "[8/35] production SRS placeholder policy"
scripts/check-production-srs-placeholder-policy.sh

echo "[9/35] deployment evidence pack"
scripts/check-deployment-evidence-pack.sh

echo "[10/35] audit readiness packet"
scripts/check-audit-readiness-packet.sh

echo "[11/35] release-candidate evidence"
scripts/check-release-candidate-evidence.sh

echo "[12/35] release checklist"
scripts/check-release-checklist.sh

echo "[13/35] GitHub release artifact tooling"
scripts/check-github-release-artifacts.sh

echo "[14/35] GitHub release page finalization"
scripts/check-github-release-page-finalization.sh

echo "[15/35] manual GitHub release publication evidence"
scripts/check-manual-github-release-publication-evidence.sh

echo "[16/35] rc2 current-main release candidate"
scripts/check-rc2-current-main-release-candidate.sh

echo "[17/35] GitHub release rc2 publication evidence"
scripts/check-github-release-rc2-publication.sh

echo "[18/35] long fuzz campaign evidence"
scripts/check-long-fuzz-campaign-evidence.sh

echo "[19/35] fuzz nightly runner policy"
scripts/check-fuzz-nightly-runner.sh

echo "[20/35] nightly fuzz smoke evidence"
scripts/check-nightly-fuzz-smoke-evidence.sh

echo "[21/35] all fuzz targets smoke evidence"
scripts/check-all-fuzz-targets-smoke-evidence.sh

echo "[22/35] fuzz crash regression suite"
scripts/check-fuzz-crash-regressions.sh

echo "[23/35] production deployment guide"
scripts/check-production-deployment-guide.sh

echo "[24/35] README star polish"
scripts/check-readme-star-polish.sh

echo "[25/35] final project positioning and roadmap"
scripts/check-final-project-positioning-and-roadmap.sh

echo "[26/35] final repository health report"
scripts/check-final-repo-health-report.sh

echo "[27/35] reviewer onboarding guide"
scripts/check-reviewer-onboarding-guide.sh

echo "[28/35] examples gallery"
scripts/check-examples-gallery.sh

echo "[29/35] paper-style technical overview"
scripts/check-paper-style-technical-overview.sh

echo "[30/35] final repo polish and freeze"
scripts/check-final-repo-polish-and-freeze.sh
echo "[31/35] visualizer screenshot assets"
scripts/check-visualizer-screenshot-assets.sh
echo "[32/35] visualizer build"
(
  cd web/visualizer
  npm ci
  npm run build
)

echo "[33/35] reject unsafe Rust"
if grep -RIn --include='*.rs' -E '\bunsafe\s*(\{|fn|impl|trait|extern)' crates fuzz; then
  echo "unsafe Rust found" >&2
  exit 1
fi

echo "[34/35] reject Number.isNaN in visualizer"
if grep -RIn 'Number\.isNaN' web/visualizer/src; then
  echo "Number.isNaN found; use the repo's explicit finite-number checks instead" >&2
  exit 1
fi

echo "[35/35] git working tree summary"
git status --short

echo "production readiness checks passed"
