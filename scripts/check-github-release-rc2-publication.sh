#!/usr/bin/env bash
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

DIR="release/publication/v0.2.0-rc.2"

test -f "$DIR/README.md"
test -f "$DIR/gh-release-view.txt"
test -f "$DIR/gh-release-view.json"
test -f "$DIR/SHA256SUMS"
test -f "$DIR/MANIFEST.txt"
test -f "$DIR/TAG_INFO.txt"

grep -q 'GitHub Release Publication Evidence: v0.2.0-rc.2' "$DIR/README.md"
grep -q 'published' "$DIR/README.md"
grep -q 'https://github.com/katastrofh/SNARK_LAB/releases/tag/v0.2.0-rc.2' "$DIR/README.md"
grep -q 'Pre-release: `true`' "$DIR/README.md"
grep -q 'Draft: `false`' "$DIR/README.md"
grep -q 'does not prove production security' "$DIR/README.md"
grep -q 'Current-main release candidate note' "$DIR/README.md"

python3 - <<'PY'
import json
from pathlib import Path

path = Path("release/publication/v0.2.0-rc.2/gh-release-view.json")
data = json.loads(path.read_text())

if data["tagName"] != "v0.2.0-rc.2":
    raise SystemExit("unexpected release tag")

if data["isDraft"] is not False:
    raise SystemExit("release must not be draft")

if data["isPrerelease"] is not True:
    raise SystemExit("release candidate must be marked as GitHub pre-release")

expected_assets = {
    "CHANGELOG.md",
    "MANIFEST.txt",
    "RELEASE_CANDIDATE_EVIDENCE.json",
    "RELEASE_CANDIDATE_EVIDENCE.md",
    "RELEASE_NOTES.md",
    "SECURITY.md",
    "SHA256SUMS",
    "SNARK_LAB-v0.2.0-rc.2.source.tar.gz",
    "SNARK_LAB-v0.2.0-rc.2.source.zip",
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

print("GitHub release rc2 publication evidence JSON is valid")
PY

if grep -RIn \
  -E 'custody-safe|mainnet-ready|guaranteed secure|externally audited|production secure' \
  "$DIR"; then
  echo "rc2 publication evidence contains unsupported security claim" >&2
  exit 1
fi

echo "GitHub release rc2 publication evidence is valid"
