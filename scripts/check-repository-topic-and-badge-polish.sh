#!/usr/bin/env bash
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

test -f .github/repository-description.txt
test -f .github/repository-topics.txt
test -f docs/repository-topic-and-badge-polish.md

grep -q 'SNARK_LAB_BADGES_V1' README.md
grep -q 'status-research%20prototype' README.md
grep -q 'release-v0.2.0--rc.1' README.md
grep -q 'language-Rust' README.md
grep -q 'fuzzing-smoke%20%2B%20regressions' README.md
grep -q 'visualizer-available' README.md
grep -q 'license-MIT%20OR%20Apache--2.0' README.md
grep -q 'Not audited production-secure software' README.md

python3 - <<'PY'
from pathlib import Path

description = Path(".github/repository-description.txt").read_text().strip()
topics = [line.strip() for line in Path(".github/repository-topics.txt").read_text().splitlines() if line.strip()]

if len(description) < 40:
    raise SystemExit("repository description is too short")

if len(description) > 350:
    raise SystemExit("repository description is too long")

if len(topics) < 10:
    raise SystemExit("too few repository topics")

if len(topics) > 20:
    raise SystemExit("GitHub supports at most 20 topics")

for topic in topics:
    if topic != topic.lower():
        raise SystemExit(f"topic must be lowercase: {topic}")
    if " " in topic:
        raise SystemExit(f"topic must not contain spaces: {topic}")
    if len(topic) > 50:
        raise SystemExit(f"topic too long: {topic}")

required = {
    "snark",
    "zk-snarks",
    "zero-knowledge",
    "sumcheck",
    "polynomial-commitments",
    "rust",
    "cryptography",
    "fuzzing",
}

missing = required - set(topics)
if missing:
    raise SystemExit(f"missing required topics: {sorted(missing)}")

print("repository topic metadata is valid")
PY

if grep -RIn \
  -E 'custody-safe|mainnet-ready|guaranteed secure|externally audited|production secure' \
  README.md .github/repository-description.txt docs/repository-topic-and-badge-polish.md; then
  echo "repository polish contains unsupported security claim" >&2
  exit 1
fi

echo "repository topic and badge polish is valid"
