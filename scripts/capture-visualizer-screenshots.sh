#!/usr/bin/env bash
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

OUT="$ROOT/docs/assets/visualizer"
PORT="${VISUALIZER_SCREENSHOT_PORT:-4173}"
HOST="127.0.0.1"
BASE_URL="http://${HOST}:${PORT}"

mkdir -p "$OUT"

find_browser() {
  if [[ -n "${BROWSER:-}" ]]; then
    command -v "$BROWSER" >/dev/null 2>&1 && { command -v "$BROWSER"; return 0; }
    echo "BROWSER is set but not executable: $BROWSER" >&2
    return 1
  fi

  for candidate in google-chrome-stable google-chrome chromium chromium-browser brave-browser microsoft-edge-stable opera; do
    if command -v "$candidate" >/dev/null 2>&1; then
      command -v "$candidate"
      return 0
    fi
  done

  return 1
}

BROWSER_BIN="$(find_browser || true)"
if [[ -z "$BROWSER_BIN" ]]; then
  echo "no Chromium-compatible browser found" >&2
  echo "install Chrome/Chromium or run with: BROWSER=/path/to/browser scripts/capture-visualizer-screenshots.sh" >&2
  exit 1
fi

cd "$ROOT/web/visualizer"
npm ci
npm run build

LOG="$ROOT/.visualizer-preview.log"
npm run preview -- --host "$HOST" --port "$PORT" > "$LOG" 2>&1 &
SERVER_PID="$!"

cleanup() {
  kill "$SERVER_PID" >/dev/null 2>&1 || true
}
trap cleanup EXIT

for _ in $(seq 1 60); do
  if curl -fsS "$BASE_URL/" >/dev/null 2>&1; then
    break
  fi
  sleep 0.25
done

if ! curl -fsS "$BASE_URL/" >/dev/null 2>&1; then
  echo "visualizer preview did not start" >&2
  cat "$LOG" >&2 || true
  exit 1
fi

USER_DATA_DIR="$(mktemp -d)"
trap 'rm -rf "$USER_DATA_DIR"; cleanup' EXIT

capture() {
  local tab="$1"
  local file="$2"
  local title="$3"

  echo "capturing ${title}: ${tab}"

  "$BROWSER_BIN" \
    --headless=new \
    --disable-gpu \
    --no-sandbox \
    --hide-scrollbars \
    --force-device-scale-factor=1 \
    --window-size=1440,1000 \
    --user-data-dir="$USER_DATA_DIR" \
    --screenshot="$OUT/$file" \
    "$BASE_URL/?tab=$tab" >/dev/null 2>&1 || {
      echo "browser screenshot failed for $tab" >&2
      exit 1
    }

  test -s "$OUT/$file"
}

capture "system" "system-flow.png" "system flow"
capture "ipa" "ipa-flow.png" "IPA flow"
capture "sumcheck" "sumcheck-flow.png" "sumcheck flow"

python3 - <<'PY'
from pathlib import Path

out = Path("docs/assets/visualizer")
for name in ["system-flow.png", "ipa-flow.png", "sumcheck-flow.png"]:
    p = out / name
    data = p.read_bytes()
    if not data.startswith(b"\x89PNG\r\n\x1a\n"):
        raise SystemExit(f"{name} is not a PNG")
    if len(data) < 10_000:
        raise SystemExit(f"{name} is suspiciously small: {len(data)} bytes")
    print(f"{name}: {len(data)} bytes")
PY

echo "visualizer screenshots captured in docs/assets/visualizer"
