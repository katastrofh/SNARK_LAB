#!/usr/bin/env bash
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

test -f FREEZE.md
test -f docs/final-repo-polish-and-freeze.md
test -f docs/post-freeze-maintenance-policy.md

grep -q 'SNARK_LAB Release-Candidate Freeze' FREEZE.md
grep -q 'Freeze status' FREEZE.md
grep -q 'v0.2.0-rc.2' FREEZE.md
grep -q 'What is frozen' FREEZE.md
grep -q 'Allowed post-freeze changes' FREEZE.md
grep -q 'Avoid post-freeze scope creep' FREEZE.md
grep -q 'protocol lab and research prototype' FREEZE.md

grep -q 'Final Repo Polish and Freeze' docs/final-repo-polish-and-freeze.md
grep -q 'Post-Freeze Maintenance Policy' docs/post-freeze-maintenance-policy.md
grep -q 'Acceptable changes' docs/post-freeze-maintenance-policy.md
grep -q 'Changes requiring justification' docs/post-freeze-maintenance-policy.md
grep -q 'Release candidate policy' docs/post-freeze-maintenance-policy.md

if grep -RIn \
  -E 'custody-safe|mainnet-ready|guaranteed secure|externally audited|production secure|production-secure release' \
  FREEZE.md docs/final-repo-polish-and-freeze.md docs/post-freeze-maintenance-policy.md; then
  echo "final repo freeze docs contain unsupported security claim" >&2
  exit 1
fi

echo "final repo polish and freeze docs are valid"
