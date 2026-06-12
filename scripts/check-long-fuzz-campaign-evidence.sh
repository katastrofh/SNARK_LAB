#!/usr/bin/env bash
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

test -x scripts/run-long-fuzz-campaign.sh
test -f fuzz/campaigns/README.md
test -f fuzz/campaigns/.gitignore
test -f fuzz/campaigns/TEMPLATE.md
test -f fuzz/campaigns/long-fuzz-campaign-manifest.example.json

bash -n scripts/run-long-fuzz-campaign.sh

python3 - <<'PY'
import json
from pathlib import Path

path = Path("fuzz/campaigns/long-fuzz-campaign-manifest.example.json")
data = json.loads(path.read_text())

required = [
    "schema",
    "status",
    "campaign_completed",
    "production_security_claim",
    "commit",
    "branch",
    "generated_utc",
    "seconds_per_target",
    "targets",
    "results",
    "notes",
]

for key in required:
    if key not in data:
        raise SystemExit(f"missing fuzz campaign manifest key: {key}")

if data["campaign_completed"] is not False:
    raise SystemExit("example fuzz campaign manifest must not claim campaign completion")

if data["production_security_claim"] is not False:
    raise SystemExit("example fuzz campaign manifest must not claim production security")

expected_targets = {
    "ipa_proof_decode",
    "ipa_integrated_opening_decode",
    "ipa_srs_file_decode",
}

if set(data["targets"]) != expected_targets:
    raise SystemExit("unexpected fuzz targets in manifest example")

print("long fuzz campaign manifest example is valid")
PY

echo "long fuzz campaign evidence process is valid"
