#!/usr/bin/env bash
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

echo "[1/3] cargo audit"
if ! command -v cargo-audit >/dev/null 2>&1; then
  cargo install cargo-audit --locked
fi
cargo audit

echo "[2/3] npm audit"
(
  cd web/visualizer
  npm ci
  npm audit --audit-level=high
)

echo "[3/3] dependency audit complete"
