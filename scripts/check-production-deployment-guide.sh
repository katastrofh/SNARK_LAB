#!/usr/bin/env bash
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

test -f docs/production-deployment-guide.md
test -f docs/operator-runbook.md
test -f deployment/templates/deployment-decision-template.json

python3 - <<'PY'
import json
from pathlib import Path

path = Path("deployment/templates/deployment-decision-template.json")
data = json.loads(path.read_text())

required = [
    "schema",
    "release",
    "tag",
    "commit",
    "decision",
    "allowed_values",
    "production_gate_passed",
    "release_artifacts_verified",
    "external_audit_completed",
    "side_channel_review_completed",
    "production_srs_evidence_complete",
    "long_fuzz_campaign_complete",
    "rollback_plan_present",
    "operator",
    "decision_utc",
    "notes",
]

for key in required:
    if key not in data:
        raise SystemExit(f"missing deployment decision key: {key}")

if data["decision"] != "reject":
    raise SystemExit("template deployment decision must default to reject")

if data["external_audit_completed"] is not False:
    raise SystemExit("template must not claim external audit completion")

if data["side_channel_review_completed"] is not False:
    raise SystemExit("template must not claim side-channel review completion")

if data["production_srs_evidence_complete"] is not False:
    raise SystemExit("template must not claim production SRS evidence completion")

if data["long_fuzz_campaign_complete"] is not False:
    raise SystemExit("template must not claim long fuzz campaign completion")

print("deployment decision template is valid")
PY

if grep -RIn \
  -E 'production-secure-approved.*true|mainnet-ready|custody-safe' \
  docs/production-deployment-guide.md docs/operator-runbook.md deployment/templates/deployment-decision-template.json; then
  echo "found unsupported production deployment claim" >&2
  exit 1
fi

echo "production deployment guide is valid"
