#!/usr/bin/env bash
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

TARGET="${1:-}"
DURATION_SECONDS="${2:-60}"

if [[ -z "$TARGET" ]]; then
  echo "usage: scripts/run-fuzz-campaign.sh <target> [duration_seconds]" >&2
  echo "" >&2
  echo "available targets:" >&2
  echo "  ipa_proof_decode" >&2
  echo "  ipa_integrated_opening_decode" >&2
  echo "  ipa_srs_file_decode" >&2
  exit 2
fi

case "$TARGET" in
  ipa_proof_decode|ipa_integrated_opening_decode|ipa_srs_file_decode)
    ;;
  *)
    echo "unknown fuzz target: $TARGET" >&2
    exit 2
    ;;
esac

if ! command -v cargo-fuzz >/dev/null 2>&1; then
  echo "cargo-fuzz is not installed." >&2
  echo "install with: cargo install cargo-fuzz" >&2
  exit 2
fi

mkdir -p fuzz/campaigns

STAMP="$(date -u +%Y%m%dT%H%M%SZ)"
OUT="fuzz/campaigns/${TARGET}-${STAMP}.log"

echo "target=$TARGET" | tee "$OUT"
echo "duration_seconds=$DURATION_SECONDS" | tee -a "$OUT"
echo "started_utc=$STAMP" | tee -a "$OUT"
echo "rustc=$(rustc --version)" | tee -a "$OUT"
echo "cargo=$(cargo --version)" | tee -a "$OUT"
echo "" | tee -a "$OUT"

cargo fuzz run "$TARGET" -- -max_total_time="$DURATION_SECONDS" 2>&1 | tee -a "$OUT"

echo "" | tee -a "$OUT"
echo "finished_utc=$(date -u +%Y%m%dT%H%M%SZ)" | tee -a "$OUT"
echo "wrote $OUT"
