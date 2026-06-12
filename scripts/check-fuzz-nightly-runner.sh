#!/usr/bin/env bash
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

test -x scripts/run-long-fuzz-campaign.sh
bash -n scripts/run-long-fuzz-campaign.sh

grep -q 'FUZZ_TOOLCHAIN="${FUZZ_TOOLCHAIN:-nightly}"' scripts/run-long-fuzz-campaign.sh
grep -q 'rustup toolchain list' scripts/run-long-fuzz-campaign.sh
grep -q 'cargo +${FUZZ_TOOLCHAIN} fuzz run' scripts/run-long-fuzz-campaign.sh
grep -q 'fuzz_toolchain' scripts/run-long-fuzz-campaign.sh
grep -q 'rustup component list --toolchain' scripts/run-long-fuzz-campaign.sh
grep -q 'rust-src component is required' scripts/run-long-fuzz-campaign.sh

echo "fuzz nightly runner policy is valid"
