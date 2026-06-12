#!/usr/bin/env bash
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

test -f srs/README.md
test -f srs/.gitignore
test -f srs/PRODUCTION_SRS_POLICY.md
test -f srs/production-srs-status.example.json

python3 - <<'PY'
import json
from pathlib import Path

path = Path("srs/production-srs-status.example.json")
data = json.loads(path.read_text())

required = [
    "schema",
    "status",
    "production_srs_artifact_committed",
    "production_srs_artifact_published",
    "production_srs_digest_published",
    "production_srs_manifest_published",
    "production_srs_transcript_published",
    "production_srs_verifier_output_present",
    "production_srs_ceremony_completed",
    "production_ready",
    "notes",
]

for key in required:
    if key not in data:
        raise SystemExit(f"missing SRS status key: {key}")

must_be_false = [
    "production_srs_artifact_committed",
    "production_srs_artifact_published",
    "production_srs_digest_published",
    "production_srs_manifest_published",
    "production_srs_transcript_published",
    "production_srs_verifier_output_present",
    "production_srs_ceremony_completed",
    "production_ready",
]

for key in must_be_false:
    if data[key] is not False:
        raise SystemExit(f"{key} must be false in example status")

if data["status"] != "no-production-srs-in-repo":
    raise SystemExit("unexpected SRS example status")

print("production SRS status example is valid")
PY

bad_files="$(
  find . \
    -path './.git' -prune -o \
    -path './target' -prune -o \
    -path './dist' -prune -o \
    -path './fuzz/campaigns/[0-9]*' -prune -o \
    -path './deployment/evidence/[0-9]*' -prune -o \
    -type f \( \
      -iname 'production.srs' -o \
      -iname 'production-srs.bin' -o \
      -iname '*production*srs*.bin' -o \
      -iname '*.ptau' -o \
      -iname '*trusted*setup*' -o \
      -iname '*powers*of*tau*' \
    \) -print
)"

if [[ -n "$bad_files" ]]; then
  echo "found forbidden production SRS placeholder/artifact files:" >&2
  echo "$bad_files" >&2
  exit 1
fi

scripts/check-srs-ceremony-spec.sh

echo "production SRS placeholder policy is valid"
