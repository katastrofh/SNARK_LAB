#!/usr/bin/env bash
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

TMP="$(mktemp)"
trap 'rm -f "$TMP"' EXIT

cargo run --quiet --locked -p snark-lab-cli -- ipa-demo > "$TMP"

diff -u test-vectors/ipa-demo-v1.txt "$TMP"

echo "public test vectors match"
