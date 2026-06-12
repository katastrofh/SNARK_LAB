#!/usr/bin/env bash
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

DIR="release/publication/v0.2.0-rc.1"

test -f "$DIR/README.md"
test -f "$DIR/gh-release-view.txt"
test -f "$DIR/gh-release-view.json"
test -f "$DIR/SHA256SUMS"
test -f "$DIR/MANIFEST.txt"
test -f "$DIR/TAG_INFO.txt"

grep -q 'GitHub Release Publication Evidence: v0.2.0-rc.1' "$DIR/README.md"
grep -q 'published' "$DIR/README.md"
grep -q 'https://github.com/katastrofh/SNARK_LAB/releases/tag/v0.2.0-rc.1' "$DIR/README.md"
grep -q 'does not prove production security' "$DIR/README.md"
grep -q 'v0.2.0-rc.2' "$DIR/README.md"

python3 - <<'PY'
import json
from pathlib import Path

path = Path("release/publication/v0.2.0-rc.1/gh-release-view.json")
data = json.loads(path.read_text())

if data["tagName"] != "v0.2.0-rc.1":
    raise SystemExit("unexpected release tag")

if data["isDraft"] is not False:
    raise SystemExit("release must not be draft")

expected_assets = {
    "CHANGELOG.md",
    "MANIFEST.txt",
    "RELEASE_CANDIDATE_EVIDENCE.json",
    "RELEASE_CANDIDATE_EVIDENCE.md",
    "RELEASE_NOTES.md",
    "SECURITY.md",
    "SHA256SUMS",
    "SNARK_LAB-v0.2.0-rc.1.source.tar.gz",
    "SNARK_LAB-v0.2.0-rc.1.source.zip",
    "SRS_MANIFEST_EXAMPLE.json",
    "TAG_INFO.txt",
}

actual_assets = {asset["name"] for asset in data["assets"]}
missing = expected_assets - actual_assets
extra = actual_assets - expected_assets

if missing:
    raise SystemExit(f"missing release assets: {sorted(missing)}")

if extra:
    raise SystemExit(f"unexpected release assets: {sorted(extra)}")

for asset in data["assets"]:
    if int(asset.get("size", 0)) <= 0:
        raise SystemExit(f"release asset has invalid size: {asset}")

print("manual GitHub release publication evidence JSON is valid")
PY

if grep -RIn \
  -E 'custody-safe|mainnet-ready|guaranteed secure|externally audited|production secure' \
  "$DIR"; then
  echo "publication evidence contains unsupported security claim" >&2
  exit 1
fi

echo "manual GitHub release publication evidence is valid"
