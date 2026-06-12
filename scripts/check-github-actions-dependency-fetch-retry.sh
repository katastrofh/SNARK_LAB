#!/usr/bin/env bash
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

test -d .github/workflows

grep -RIn "FORCE_JAVASCRIPT_ACTIONS_TO_NODE24: true" .github/workflows >/dev/null
grep -RIn "Pre-fetch Rust dependencies with retry" .github/workflows >/dev/null
grep -RIn "cargo fetch --locked" .github/workflows >/dev/null

if grep -RIn \
  --include='*.yml' \
  --include='*.yaml' \
  "cargo metadata.*--workspace" \
  .github/workflows; then
  echo "cargo metadata does not support --workspace" >&2
  exit 1
fi

echo "GitHub Actions dependency fetch retry policy is valid"
