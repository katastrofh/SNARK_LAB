#!/usr/bin/env bash
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

echo "[fuzz] compile all fuzz targets"
cargo check --locked --manifest-path fuzz/Cargo.toml --bins

echo "[fuzz] targets compile"
