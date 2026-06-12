#!/usr/bin/env bash
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

test -x scripts/build-github-release-artifacts.sh

bash -n scripts/build-github-release-artifacts.sh

if [[ -f release/GITHUB_RELEASE_DRAFT_v0.2.0-rc.1.md ]]; then
  grep -q "v0.2.0-rc.1" release/GITHUB_RELEASE_DRAFT_v0.2.0-rc.1.md
  grep -q "not production-secure" release/GITHUB_RELEASE_DRAFT_v0.2.0-rc.1.md
fi

echo "GitHub release artifact tooling is valid"
