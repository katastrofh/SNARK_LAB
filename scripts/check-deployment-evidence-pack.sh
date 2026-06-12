#!/usr/bin/env bash
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

test -x scripts/collect-deployment-evidence.sh
test -f deployment/README.md
test -f deployment/evidence/README.md
test -f deployment/evidence/.gitignore
test -f deployment/templates/release-evidence-template.md
test -f deployment/templates/release-attestation-template.json

python3 - <<'PY'
import json
from pathlib import Path

template = Path("deployment/templates/release-attestation-template.json")
data = json.loads(template.read_text())

required = [
    "schema",
    "status",
    "release",
    "tag",
    "commit",
    "generated_utc",
    "production_gate_passed",
    "public_vectors_passed",
    "srs_manifest_verified",
    "strict_production_srs_manifest_verified",
    "external_audit_completed",
    "side_channel_review_completed",
    "artifact_sha256s",
]

for key in required:
    if key not in data:
        raise SystemExit(f"missing attestation key: {key}")

print("deployment evidence templates are valid")
PY

echo "deployment evidence pack process is valid"
