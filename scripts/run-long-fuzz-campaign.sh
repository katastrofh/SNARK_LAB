#!/usr/bin/env bash
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

DURATION_PER_TARGET="${FUZZ_SECONDS_PER_TARGET:-300}"
STAMP="$(date -u +%Y%m%dT%H%M%SZ)"
OUT="fuzz/campaigns/${STAMP}"

TARGETS=(
  ipa_proof_decode
  ipa_integrated_opening_decode
  ipa_srs_file_decode
)

if ! command -v cargo-fuzz >/dev/null 2>&1 && ! cargo fuzz --help >/dev/null 2>&1; then
  echo "cargo-fuzz is not installed" >&2
  echo "install with: cargo install cargo-fuzz" >&2
  exit 1
fi

mkdir -p "$OUT"

{
  echo "# Long Fuzz Campaign Report"
  echo
  echo "## Identity"
  echo
  echo "- Campaign: ${STAMP}"
  echo "- Date UTC: $(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "- Commit: $(git rev-parse HEAD)"
  echo "- Branch: $(git branch --show-current)"
  echo
  echo "## Environment"
  echo
  echo "- uname: $(uname -a)"
  echo "- rustc: $(rustc --version 2>/dev/null || true)"
  echo "- cargo: $(cargo --version 2>/dev/null || true)"
  echo "- seconds per target: ${DURATION_PER_TARGET}"
  echo
} > "$OUT/SUMMARY.md"

for target in "${TARGETS[@]}"; do
  mkdir -p "$OUT/$target/artifacts"

  echo "running fuzz target: $target"
  echo "duration: ${DURATION_PER_TARGET}s"

  (
    cd "$ROOT/fuzz"
    cargo fuzz run "$target" -- \
      -max_total_time="${DURATION_PER_TARGET}" \
      -artifact_prefix="$ROOT/$OUT/$target/artifacts/"
  ) > "$OUT/$target.log" 2>&1 || {
    echo "target failed: $target" | tee -a "$OUT/SUMMARY.md"
    echo "see: $OUT/$target.log" | tee -a "$OUT/SUMMARY.md"
    exit 1
  }

  echo "- ${target}: complete" >> "$OUT/SUMMARY.md"
done

{
  echo
  echo "## SHA-256"
  echo
  find "$OUT" -type f ! -name SHA256SUMS -print0 | sort -z | xargs -0 sha256sum
} > "$OUT/SHA256SUMS"

python3 - <<PY
import json
from pathlib import Path

out = Path("${OUT}")
manifest = {
    "schema": "snark-lab-long-fuzz-campaign-v1",
    "status": "completed-local-campaign",
    "campaign_completed": True,
    "production_security_claim": False,
    "commit": "$(git rev-parse HEAD)",
    "branch": "$(git branch --show-current)",
    "generated_utc": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
    "seconds_per_target": int("${DURATION_PER_TARGET}"),
    "targets": ["ipa_proof_decode", "ipa_integrated_opening_decode", "ipa_srs_file_decode"],
    "results": {
        "crashes": "inspect logs",
        "timeouts": "inspect logs",
        "ooms": "inspect logs",
        "regressions_added": "manual follow-up required"
    },
    "notes": "Generated campaign evidence. Production security still requires audit, side-channel review, and production SRS evidence."
}
(out / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\\n")
PY

echo "fuzz campaign evidence written to $OUT"
