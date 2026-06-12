#!/usr/bin/env bash
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

BODY="release/GITHUB_RELEASE_PAGE_v0.2.0-rc.1.md"

test -f "$BODY"
test -x scripts/print-github-release-command.sh

grep -q 'SNARK_LAB v0.2.0-rc.1' "$BODY"
grep -q 'not production-secure software' "$BODY"
grep -q 'Release assets to attach' "$BODY"
grep -q 'scripts/build-github-release-artifacts.sh v0.2.0-rc.1' "$BODY"
grep -q 'sha256sum -c SHA256SUMS' "$BODY"
grep -q 'Known limitations' "$BODY"
grep -q 'Do not use it as production cryptographic infrastructure' "$BODY"

scripts/print-github-release-command.sh v0.2.0-rc.1 >/tmp/snark_lab_release_command.txt

grep -q 'gh release create v0.2.0-rc.1' /tmp/snark_lab_release_command.txt
grep -q 'SNARK_LAB-v0.2.0-rc.1.source.tar.gz' /tmp/snark_lab_release_command.txt
grep -q 'SHA256SUMS' /tmp/snark_lab_release_command.txt
grep -q 'RELEASE_CANDIDATE_EVIDENCE.json' /tmp/snark_lab_release_command.txt

if grep -RIn \
  -E 'custody-safe|mainnet-ready|guaranteed secure|externally audited|production secure' \
  "$BODY" scripts/print-github-release-command.sh; then
  echo "GitHub release finalization contains unsupported security claim" >&2
  exit 1
fi

echo "GitHub release page finalization is valid"
