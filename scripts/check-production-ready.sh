#!/usr/bin/env bash
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

echo "[1/34] cargo fmt"
cargo fmt --all -- --check

echo "[2/34] cargo clippy"
cargo clippy --locked --workspace --all-targets -- -D warnings

echo "[3/34] cargo test"
cargo test --locked --workspace

echo "[4/34] fuzz target build"
scripts/check-fuzz-targets.sh

echo "[5/34] public test vectors"
scripts/check-test-vectors.sh

echo "[6/34] SRS ceremony spec"
scripts/check-srs-ceremony-spec.sh

echo "[7/34] production SRS placeholder policy"
scripts/check-production-srs-placeholder-policy.sh

echo "[8/34] deployment evidence pack"
scripts/check-deployment-evidence-pack.sh

echo "[9/34] audit readiness packet"
scripts/check-audit-readiness-packet.sh

echo "[10/34] release-candidate evidence"
scripts/check-release-candidate-evidence.sh

echo "[11/34] release checklist"
scripts/check-release-checklist.sh

echo "[12/34] GitHub release artifact tooling"
scripts/check-github-release-artifacts.sh

echo "[13/34] GitHub release page finalization"
scripts/check-github-release-page-finalization.sh

echo "[14/34] manual GitHub release publication evidence"
scripts/check-manual-github-release-publication-evidence.sh

echo "[15/34] rc2 current-main release candidate"
scripts/check-rc2-current-main-release-candidate.sh

echo "[16/34] GitHub release rc2 publication evidence"
scripts/check-github-release-rc2-publication.sh

echo "[17/34] long fuzz campaign evidence"
scripts/check-long-fuzz-campaign-evidence.sh

echo "[18/34] fuzz nightly runner policy"
scripts/check-fuzz-nightly-runner.sh

echo "[19/34] nightly fuzz smoke evidence"
scripts/check-nightly-fuzz-smoke-evidence.sh

echo "[20/34] all fuzz targets smoke evidence"
scripts/check-all-fuzz-targets-smoke-evidence.sh

echo "[21/34] fuzz crash regression suite"
scripts/check-fuzz-crash-regressions.sh

echo "[22/34] production deployment guide"
scripts/check-production-deployment-guide.sh

echo "[23/34] README star polish"
scripts/check-readme-star-polish.sh

echo "[24/34] final project positioning and roadmap"
scripts/check-final-project-positioning-and-roadmap.sh

echo "[25/34] final repository health report"
scripts/check-final-repo-health-report.sh

echo "[26/34] reviewer onboarding guide"
scripts/check-reviewer-onboarding-guide.sh

echo "[27/34] examples gallery"
scripts/check-examples-gallery.sh

echo "[28/34] paper-style technical overview"
scripts/check-paper-style-technical-overview.sh

echo "[29/34] final repo polish and freeze"
scripts/check-final-repo-polish-and-freeze.sh
echo "[30/34] visualizer screenshot assets"
scripts/check-visualizer-screenshot-assets.sh
echo "[31/34] visualizer build"
(
  cd web/visualizer
  npm ci
  npm run build
)

echo "[32/34] reject unsafe Rust"
if grep -RIn --include='*.rs' -E '\bunsafe\s*(\{|fn|impl|trait|extern)' crates fuzz; then
  echo "unsafe Rust found" >&2
  exit 1
fi

echo "[33/34] reject Number.isNaN in visualizer"
if grep -RIn 'Number\.isNaN' web/visualizer/src; then
  echo "Number.isNaN found; use the repo's explicit finite-number checks instead" >&2
  exit 1
fi

echo "[34/34] git working tree summary"
git status --short

echo "production readiness checks passed"
