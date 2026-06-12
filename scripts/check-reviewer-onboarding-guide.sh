#!/usr/bin/env bash
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

test -f docs/reviewer-onboarding-guide.md
test -f REVIEWERS.md
test -f docs/reviewer-onboarding-branch-notes.md

grep -q 'Reviewer Onboarding Guide' docs/reviewer-onboarding-guide.md
grep -q 'First five minutes' docs/reviewer-onboarding-guide.md
grep -q 'scripts/check-production-ready.sh' docs/reviewer-onboarding-guide.md
grep -q 'Protocol areas to review' docs/reviewer-onboarding-guide.md
grep -q 'Evidence areas to inspect' docs/reviewer-onboarding-guide.md
grep -q 'v0.2.0-rc.2' docs/reviewer-onboarding-guide.md
grep -q 'Security boundary' docs/reviewer-onboarding-guide.md

grep -q 'Reviewer Quick Start' REVIEWERS.md
grep -q 'Run the main gate' REVIEWERS.md
grep -q 'Inspect releases' REVIEWERS.md
grep -q 'Inspect protocol evidence' REVIEWERS.md
grep -q 'not deployment-grade cryptographic infrastructure' REVIEWERS.md

grep -q 'Reviewer Onboarding Branch Notes' docs/reviewer-onboarding-branch-notes.md

if grep -RIn \
  -E 'custody-safe|mainnet-ready|guaranteed secure|externally audited|production secure|production-secure release' \
  docs/reviewer-onboarding-guide.md REVIEWERS.md docs/reviewer-onboarding-branch-notes.md; then
  echo "reviewer onboarding guide contains unsupported security claim" >&2
  exit 1
fi

echo "reviewer onboarding guide is valid"
