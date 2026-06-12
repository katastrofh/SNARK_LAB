#!/usr/bin/env bash
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

DIR="fuzz/smoke-evidence/v0.2.0-rc.1/all-targets"

test -f "$DIR/README.md"
test -f "$DIR/manifest.json"

for target in ipa_proof_decode ipa_integrated_opening_decode ipa_srs_file_decode; do
  test -f "$DIR/${target}.tail.log"
  grep -q "DONE" "$DIR/${target}.tail.log"
  grep -q "Done " "$DIR/${target}.tail.log"
done

python3 - <<'PY'
import json
from pathlib import Path

path = Path("fuzz/smoke-evidence/v0.2.0-rc.1/all-targets/manifest.json")
data = json.loads(path.read_text())

required = [
    "schema",
    "status",
    "targets",
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
        raise SystemExit(f"missing all-target smoke evidence key: {key}")

if data["status"] != "smoke-complete":
    raise SystemExit("all-target smoke evidence status must be smoke-complete")

if data["production_security_claim"] is not False:
    raise SystemExit("smoke evidence must not claim production security")

if data["long_campaign_claim"] is not False:
    raise SystemExit("smoke evidence must not claim long campaign completion")

expected = {
    "ipa_proof_decode",
    "ipa_integrated_opening_decode",
    "ipa_srs_file_decode",
}

actual = {entry.get("target") for entry in data["targets"]}
if actual != expected:
    raise SystemExit(f"unexpected smoke targets: {actual}")

for entry in data["targets"]:
    if entry.get("status") != "smoke-complete":
        raise SystemExit(f"target not smoke-complete: {entry}")
    if entry.get("runs") is not None and entry["runs"] <= 0:
        raise SystemExit(f"target has invalid run count: {entry}")
    if entry.get("seconds") is not None and entry["seconds"] <= 0:
        raise SystemExit(f"target has invalid duration: {entry}")

print("all fuzz targets smoke evidence manifest is valid")
PY

echo "all fuzz targets smoke evidence is valid"
