#!/usr/bin/env bash
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

test -f examples/README.md
test -f examples/sumcheck-example.md
test -f examples/zerocheck-example.md
test -f examples/permcheck-example.md
test -f examples/ipa-example.md
test -f examples/release-evidence-example.md
test -f examples/visualizer-example.md
test -f docs/examples-gallery.md

grep -q 'SNARK_LAB Examples Gallery' examples/README.md
grep -q 'Sumcheck Example' examples/sumcheck-example.md
grep -q 'Zerocheck Example' examples/zerocheck-example.md
grep -q 'PermCheck Example' examples/permcheck-example.md
grep -q 'IPA Polynomial Commitment Example' examples/ipa-example.md
grep -q 'Release Evidence Example' examples/release-evidence-example.md
grep -q 'Visualizer Example' examples/visualizer-example.md
grep -q 'Examples Gallery' docs/examples-gallery.md

grep -q 'cargo test -p sumcheck' examples/sumcheck-example.md
grep -q 'cargo test -p zerocheck' examples/zerocheck-example.md
grep -q 'cargo test -p permcheck' examples/permcheck-example.md
grep -q 'cargo test -p snark_lab_oracle ipa' examples/ipa-example.md
grep -q 'scripts/check-production-ready.sh' examples/release-evidence-example.md
grep -q 'npm run dev' examples/visualizer-example.md

if grep -RIn \
  -E 'custody-safe|mainnet-ready|guaranteed secure|externally audited|production secure|production-secure release' \
  examples docs/examples-gallery.md; then
  echo "examples gallery contains unsupported security claim" >&2
  exit 1
fi

echo "examples gallery is valid"
