#!/usr/bin/env bash
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

mkdir -p test-vectors

cargo run --quiet --locked -p snark-lab-cli -- ipa-demo > test-vectors/ipa-demo-v1.txt

echo "generated test-vectors/ipa-demo-v1.txt"
