#!/usr/bin/env bash
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

test -f docs/project-positioning.md
test -f ROADMAP.md
test -f docs/final-project-positioning-and-roadmap.md

grep -q 'SNARK_LAB is a Rust protocol lab' docs/project-positioning.md
grep -q 'What this project is' docs/project-positioning.md
grep -q 'What this project is not' docs/project-positioning.md
grep -q 'Research prototype. Not audited production-secure software.' docs/project-positioning.md
grep -q 'Current maturity' docs/project-positioning.md

grep -q 'SNARK_LAB Roadmap' ROADMAP.md
grep -q 'Current release-candidate track' ROADMAP.md
grep -q 'Near-term engineering work' ROADMAP.md
grep -q 'Research work' ROADMAP.md
grep -q 'Security-review work' ROADMAP.md
grep -q 'Non-goals' ROADMAP.md

grep -q 'Final Project Positioning and Roadmap' docs/final-project-positioning-and-roadmap.md

if grep -RIn \
  -E 'custody-safe|mainnet-ready|guaranteed secure|externally audited|production secure|production-secure release' \
  docs/project-positioning.md ROADMAP.md docs/final-project-positioning-and-roadmap.md; then
  echo "project positioning contains unsupported security claim" >&2
  exit 1
fi

echo "final project positioning and roadmap are valid"
