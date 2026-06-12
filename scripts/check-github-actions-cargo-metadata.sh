#!/usr/bin/env bash
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

if grep -RIn \
  --include='*.yml' \
  --include='*.yaml' \
  "cargo metadata.*--workspace" \
  .github/workflows; then
  echo "cargo metadata does not support --workspace" >&2
  exit 1
fi

cargo metadata --locked --format-version 1 > /dev/null

echo "GitHub Actions cargo metadata usage is valid"
