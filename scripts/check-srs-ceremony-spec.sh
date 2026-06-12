#!/usr/bin/env bash
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

python3 scripts/verify-srs-ceremony-manifest.py ceremony/production-srs-manifest.example.json

echo "SRS ceremony manifest example is valid"
