#!/usr/bin/env bash
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

test -d .github/workflows
test -f scripts/audit-dependencies.sh

if grep -RIn \
  --include='*.yml' \
  --include='*.yaml' \
  --fixed-strings \
  "cargo audit --locked" \
  .github/workflows; then
  echo "cargo audit in this CI image does not support --locked" >&2
  exit 1
fi

if grep -In \
  --fixed-strings \
  "cargo audit --locked" \
  scripts/audit-dependencies.sh; then
  echo "cargo audit in the dependency audit script should not use --locked" >&2
  exit 1
fi

grep -RIn \
  --include='*.yml' \
  --include='*.yaml' \
  --fixed-strings \
  "cargo audit" \
  .github/workflows >/dev/null

grep -q \
  --fixed-strings \
  "cargo audit" \
  scripts/audit-dependencies.sh

echo "GitHub Actions cargo audit usage is valid"
