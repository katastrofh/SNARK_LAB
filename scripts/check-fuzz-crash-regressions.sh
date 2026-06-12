#!/usr/bin/env bash
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

test -f fuzz/regressions/README.md
test -f fuzz/regressions/ipa_proof_decode/capacity-overflow-20260612.json
test -f crates/oracle/tests/ipa_fuzz_crash_regressions.rs

python3 - <<'PY'
import base64
import json
from pathlib import Path

path = Path("fuzz/regressions/ipa_proof_decode/capacity-overflow-20260612.json")
data = json.loads(path.read_text())

required = [
    "schema",
    "target",
    "date_utc",
    "status",
    "bug_class",
    "expected_behavior",
    "expected_error",
    "source",
    "base64",
    "notes",
]

for key in required:
    if key not in data:
        raise SystemExit(f"missing fuzz regression key: {key}")

if data["schema"] != "snark-lab-fuzz-regression-v1":
    raise SystemExit("unexpected fuzz regression schema")

if data["target"] != "ipa_proof_decode":
    raise SystemExit("unexpected fuzz regression target")

if data["status"] != "fixed":
    raise SystemExit("fuzz regression must be marked fixed")

if data["expected_behavior"] != "decode-error-no-panic":
    raise SystemExit("unexpected fuzz regression expected behavior")

if data["expected_error"] != "LengthOverflow":
    raise SystemExit("unexpected fuzz regression expected error")

raw = base64.b64decode(data["base64"])

if not raw.startswith(b"SL-IPA-PROOF1"):
    raise SystemExit("regression input does not start with IPA proof magic")

if len(raw) < 32:
    raise SystemExit("regression input is unexpectedly short")

print("fuzz regression metadata is valid")
PY

cargo test -p snark-lab-oracle --test ipa_fuzz_crash_regressions

echo "fuzz crash regression suite is valid"
