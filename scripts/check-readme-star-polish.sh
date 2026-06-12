#!/usr/bin/env bash
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

test -f README.md

grep -q 'SNARK_LAB_STAR_POLISH_V1' README.md
grep -q '## Why this repository matters' README.md
grep -q '## Current status' README.md
grep -q '## What this is' README.md
grep -q '## What this is not' README.md
grep -q '## Quickstart' README.md
grep -q '## Protocol map' README.md
grep -q '## Evidence and hardening' README.md
grep -q '## Suggested reading order' README.md
grep -q 'v0.2.0-rc.1' README.md
grep -q 'not production-secure' README.md
grep -q 'scripts/check-production-ready.sh' README.md
grep -q 'web/visualizer' README.md
grep -q 'fuzz/regressions' README.md

if grep -RIn \
  -E 'audited production SNARK library|safe for custody|guaranteed secure' \
  README.md; then
  echo "README contains unsupported security claim" >&2
  exit 1
fi

echo "README star polish is valid"
