#!/usr/bin/env bash
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

test -f release-candidates/README.md
test -x scripts/summarize-latest-deployment-evidence.py

if [[ -f release-candidates/LATEST.json ]]; then
  python3 - <<'PY'
import json
from pathlib import Path

path = Path("release-candidates/LATEST.json")
data = json.loads(path.read_text())

required = [
    "schema",
    "status",
    "evidence_pack",
    "branch_under_evidence",
    "commit_under_evidence",
    "git_status_clean",
    "production_gate_passed",
    "public_vectors_passed",
    "srs_manifest_example_passed",
    "fuzz_targets_compile_passed",
    "external_audit_completed",
    "side_channel_review_completed",
    "production_srs_ceremony_completed",
    "production_secure",
]

for key in required:
    if key not in data:
        raise SystemExit(f"missing release-candidate summary key: {key}")

must_be_false = [
    "external_audit_completed",
    "side_channel_review_completed",
    "production_srs_ceremony_completed",
    "production_secure",
]

for key in must_be_false:
    if data[key] is not False:
        raise SystemExit(f"{key} must be false until real evidence exists")

print("release-candidate evidence summary is valid")
PY
fi

echo "release-candidate evidence tooling is valid"
