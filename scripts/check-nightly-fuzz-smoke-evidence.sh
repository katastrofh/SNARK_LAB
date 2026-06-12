#!/usr/bin/env bash
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

test -f fuzz/smoke-evidence/v0.2.0-rc.1/ipa_proof_decode_smoke.md
test -f fuzz/smoke-evidence/v0.2.0-rc.1/manifest.json
test -f fuzz/smoke-evidence/v0.2.0-rc.1/ipa_proof_decode_smoke_tail.log

python3 - <<'PY'
import json
from pathlib import Path

path = Path("fuzz/smoke-evidence/v0.2.0-rc.1/manifest.json")
data = json.loads(path.read_text())

required = [
    "schema",
    "status",
    "target",
    "command",
    "runs",
    "seconds",
    "toolchain",
    "commit",
    "branch",
    "generated_utc",
    "production_security_claim",
    "long_campaign_claim",
    "notes",
]

for key in required:
    if key not in data:
        raise SystemExit(f"missing smoke evidence key: {key}")

if data["status"] != "smoke-complete":
    raise SystemExit("smoke evidence status must be smoke-complete")

if data["target"] != "ipa_proof_decode":
    raise SystemExit("unexpected smoke evidence target")

if data["production_security_claim"] is not False:
    raise SystemExit("smoke evidence must not claim production security")

if data["long_campaign_claim"] is not False:
    raise SystemExit("smoke evidence must not claim long campaign completion")

if data["runs"] is not None and data["runs"] <= 0:
    raise SystemExit("smoke evidence run count must be positive")

if data["seconds"] is not None and data["seconds"] <= 0:
    raise SystemExit("smoke evidence duration must be positive")

print("nightly fuzz smoke evidence manifest is valid")
PY

grep -q "DONE" fuzz/smoke-evidence/v0.2.0-rc.1/ipa_proof_decode_smoke_tail.log
grep -q "Done " fuzz/smoke-evidence/v0.2.0-rc.1/ipa_proof_decode_smoke_tail.log

echo "nightly fuzz smoke evidence is valid"
