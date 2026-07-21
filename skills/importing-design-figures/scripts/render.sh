#!/usr/bin/env bash
# render.sh — one command: launch headless Chrome, serve the working dir, clip the
# figure plate to a PNG, and clean up. Wraps shoot.mjs so a caller never has to
# manage the browser or a server by hand.
#
# Usage:
#   bash render.sh <workdir> <dcfile> <out.png> [scale] [css-selector] [field-color] \
#     [target-width-mm] [fonts]
#     workdir          dir containing the .dc.html AND its support.js (for runtime assets)
#     dcfile           filename of the .dc.html inside workdir (spaces are fine)
#     out.png          output path for the rendered plate
#     scale            device pixel ratio; 3 => ~430 DPI at the project's target width
#                       (default 3)
#     selector         optional CSS selector for the plate (default: auto-detect). Passing
#                       an explicit selector is the reliable path; see references/pipeline.md.
#     field            optional CSS colour of the plate background. Omit and shoot.mjs reads
#                       the --field token from :root automatically.
#     target-width-mm  optional artboard width in mm (from the project's config.md); used
#                       only to print the true placement height. Omit to print relative
#                       aspect only.
#     fonts            optional comma-separated font families to probe for substitution
#                       (from the project's design tokens). Omit to skip the probe.
#
# Requirements (see scripts/SETUP.md): only Google Chrome.app + Node >= 20. No
# Playwright/Puppeteer/npm/pip — the harness drives Chrome directly over CDP.
#   - Google Chrome at /Applications/Google Chrome.app
#   - Node >= 20 (built-in fetch + WebSocket)
#   - python3 (static file server)
#   - Network access to unpkg.com for runtime-generated assets (React/Babel)
set -euo pipefail

WORKDIR="${1:?workdir}"; DCFILE="${2:?dcfile}"; OUT="${3:?out.png}"
SCALE="${4:-3}"; SELECTOR="${5:-}"; FIELD="${6:-}"
TARGET_WIDTH_MM="${7:-}"; FONTS="${8:-}"
HERE="$(cd "$(dirname "$0")" && pwd)"
CHROME="/Applications/Google Chrome.app/Contents/MacOS/Google Chrome"
PORT=8123; DBG=9222
UDD="$(mktemp -d)"

"$CHROME" --headless=new --disable-gpu --hide-scrollbars \
  --remote-debugging-port=$DBG --user-data-dir="$UDD" \
  --window-size=1600,2600 about:blank >/tmp/cdf-chrome.log 2>&1 &
CHROME_PID=$!
python3 -m http.server $PORT --directory "$WORKDIR" >/tmp/cdf-httpd.log 2>&1 &
HTTP_PID=$!
disown $CHROME_PID $HTTP_PID 2>/dev/null || true   # keep job control quiet on cleanup
cleanup(){ kill $CHROME_PID $HTTP_PID 2>/dev/null || true; rm -rf "$UDD" 2>/dev/null || true; }
trap cleanup EXIT

# URL-encode spaces (component files are named e.g. "Figure 1 Overview.dc.html")
ENC="${DCFILE// /%20}"
# wait for readiness without a foreground `sleep` (blocked in this harness)
curl -s --retry 40 --retry-delay 1 --retry-all-errors "http://127.0.0.1:$DBG/json" >/dev/null
curl -s --retry 40 --retry-delay 1 --retry-connrefused "http://127.0.0.1:$PORT/$ENC" >/dev/null

ARGS=(--url "http://127.0.0.1:$PORT/$ENC" --out "$OUT" --scale "$SCALE")
[ -n "$SELECTOR" ] && ARGS+=(--selector "$SELECTOR")
[ -n "$FIELD" ] && ARGS+=(--field "$FIELD")
[ -n "$TARGET_WIDTH_MM" ] && ARGS+=(--target-width-mm "$TARGET_WIDTH_MM")
[ -n "$FONTS" ] && ARGS+=(--fonts "$FONTS")
node "$HERE/shoot.mjs" "${ARGS[@]}"
