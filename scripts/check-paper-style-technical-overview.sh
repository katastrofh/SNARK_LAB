#!/usr/bin/env bash
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

test -f docs/paper-style-technical-overview.md
test -f docs/protocol-stack-summary.md
test -f docs/paper-style-technical-overview-notes.md

grep -q 'Paper-Style Technical Overview' docs/paper-style-technical-overview.md
grep -q 'Abstract' docs/paper-style-technical-overview.md
grep -q 'Motivation' docs/paper-style-technical-overview.md
grep -q 'Protocol stack' docs/paper-style-technical-overview.md
grep -q 'IPA commitment path' docs/paper-style-technical-overview.md
grep -q 'Evidence model' docs/paper-style-technical-overview.md
grep -q 'Release model' docs/paper-style-technical-overview.md
grep -q 'Visualizer' docs/paper-style-technical-overview.md
grep -q 'Limitations' docs/paper-style-technical-overview.md

grep -q 'Protocol Stack Summary' docs/protocol-stack-summary.md
grep -q 'Layer 1' docs/protocol-stack-summary.md
grep -q 'Layer 7' docs/protocol-stack-summary.md

grep -q 'Paper-Style Technical Overview Notes' docs/paper-style-technical-overview-notes.md

if grep -RIn \
  -E 'custody-safe|mainnet-ready|guaranteed secure|externally audited|production secure|production-secure release' \
  docs/paper-style-technical-overview.md docs/protocol-stack-summary.md docs/paper-style-technical-overview-notes.md; then
  echo "paper-style technical overview contains unsupported security claim" >&2
  exit 1
fi

echo "paper-style technical overview is valid"
