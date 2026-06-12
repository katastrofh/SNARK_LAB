#!/usr/bin/env bash
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

test -f release/v0.2.0-rc.2.md
test -f release/GITHUB_RELEASE_PAGE_v0.2.0-rc.2.md

grep -q 'SNARK_LAB v0.2.0-rc.2' release/v0.2.0-rc.2.md
grep -q 'release candidate from current main' release/v0.2.0-rc.2.md
grep -q 'not production-secure software' release/v0.2.0-rc.2.md
grep -q 'Added since v0.2.0-rc.1' release/v0.2.0-rc.2.md
grep -q 'Fuzz crash regression suite' release/v0.2.0-rc.2.md
grep -q 'Visualizer screenshot assets' release/v0.2.0-rc.2.md

grep -q 'SNARK_LAB v0.2.0-rc.2' release/GITHUB_RELEASE_PAGE_v0.2.0-rc.2.md
grep -q 'scripts/build-github-release-artifacts.sh v0.2.0-rc.2' release/GITHUB_RELEASE_PAGE_v0.2.0-rc.2.md
grep -q 'SNARK_LAB-v0.2.0-rc.2.source.tar.gz' release/GITHUB_RELEASE_PAGE_v0.2.0-rc.2.md
grep -q 'sha256sum -c SHA256SUMS' release/GITHUB_RELEASE_PAGE_v0.2.0-rc.2.md
grep -q 'Do not use it as production cryptographic infrastructure' release/GITHUB_RELEASE_PAGE_v0.2.0-rc.2.md

if grep -RIn \
  -E 'custody-safe|mainnet-ready|guaranteed secure|externally audited|production secure' \
  release/v0.2.0-rc.2.md release/GITHUB_RELEASE_PAGE_v0.2.0-rc.2.md; then
  echo "rc2 release candidate docs contain unsupported security claim" >&2
  exit 1
fi

echo "rc2 current-main release candidate docs are valid"
