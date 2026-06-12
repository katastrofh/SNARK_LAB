#!/usr/bin/env bash
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

required_files=(
  audits/README.md
  audits/scope.md
  audits/triage-policy.md
  audits/remediation-log.md
  audits/audit-status.example.json
  audits/templates/finding-template.md
  audits/templates/audit-request-template.md
  audits/packet/README.md
)

for file in "${required_files[@]}"; do
  test -f "$file"
done

python3 - <<'PY'
import json
from pathlib import Path

path = Path("audits/audit-status.example.json")
data = json.loads(path.read_text())

required = [
    "schema",
    "status",
    "repository",
    "commit_under_review",
    "external_audit_completed",
    "side_channel_review_completed",
    "production_srs_ceremony_completed",
    "production_deployment_approved",
    "production_secure",
    "auditor",
    "findings",
    "evidence",
]

for key in required:
    if key not in data:
        raise SystemExit(f"missing audit status key: {key}")

must_be_false = [
    "external_audit_completed",
    "side_channel_review_completed",
    "production_srs_ceremony_completed",
    "production_deployment_approved",
    "production_secure",
]

for key in must_be_false:
    if data[key] is not False:
        raise SystemExit(f"{key} must be false in the example audit status")

if data["status"] != "audit-ready-not-audited":
    raise SystemExit("example audit status must be audit-ready-not-audited")

print("audit status example is valid")
PY

if grep -RIn \
  -E 'external audit completed: true|production_secure: true|production-secure release approved' \
  audits docs SECURITY.md README.md \
  --exclude='audit-status.example.json'; then
  echo "found unsupported production/audit claim" >&2
  exit 1
fi

echo "audit readiness packet is valid"
