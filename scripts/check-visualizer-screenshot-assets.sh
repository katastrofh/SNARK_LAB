#!/usr/bin/env bash
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

for file in \
  docs/assets/visualizer/system-flow.png \
  docs/assets/visualizer/ipa-flow.png \
  docs/assets/visualizer/sumcheck-flow.png
do
  test -f "$file"
  test -s "$file"
done

python3 - <<'PY'
from pathlib import Path

files = [
    "docs/assets/visualizer/system-flow.png",
    "docs/assets/visualizer/ipa-flow.png",
    "docs/assets/visualizer/sumcheck-flow.png",
]

for name in files:
    p = Path(name)
    data = p.read_bytes()
    if not data.startswith(b"\x89PNG\r\n\x1a\n"):
        raise SystemExit(f"{name} is not a valid PNG")
    if len(data) < 10_000:
        raise SystemExit(f"{name} is too small to be a useful screenshot: {len(data)} bytes")

print("visualizer screenshot PNG assets are valid")
PY

grep -q 'docs/assets/visualizer/system-flow.png' README.md
grep -q 'docs/assets/visualizer/ipa-flow.png' README.md
grep -q 'docs/assets/visualizer/sumcheck-flow.png' README.md

echo "visualizer screenshot assets are valid"
