#!/usr/bin/env bash
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

test -f docs/final-repo-health-report.md
test -f docs/final-repo-health-report-notes.md

grep -q 'Final Repository Health Report' docs/final-repo-health-report.md
grep -q 'Published release candidates' docs/final-repo-health-report.md
grep -q 'v0.2.0-rc.1' docs/final-repo-health-report.md
grep -q 'v0.2.0-rc.2' docs/final-repo-health-report.md
grep -q 'Evidence stack' docs/final-repo-health-report.md
grep -q 'Automated gate' docs/final-repo-health-report.md
grep -q 'scripts/check-production-ready.sh' docs/final-repo-health-report.md
grep -q 'Remaining blockers before stronger security claims' docs/final-repo-health-report.md
grep -q 'Recommended next work' docs/final-repo-health-report.md
grep -q 'does not prove cryptographic deployment readiness' docs/final-repo-health-report.md

grep -q 'Final Repository Health Report Notes' docs/final-repo-health-report-notes.md
grep -q 'repository-level evidence' docs/final-repo-health-report-notes.md

if grep -RIn \
  -E 'custody-safe|mainnet-ready|guaranteed secure|externally audited|production secure|production-secure release' \
  docs/final-repo-health-report.md docs/final-repo-health-report-notes.md; then
  echo "final repo health report contains unsupported security claim" >&2
  exit 1
fi

echo "final repo health report is valid"
