#!/usr/bin/env bash
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

python3 scripts/check-release-checklist.py

echo "release checklist process is valid"
