# Setup and requirements

This harness is deliberately **zero-dependency beyond Google Chrome**. It drives a
already-installed Chrome directly over the DevTools Protocol from Node's built-ins, so
there is nothing to `npm install` or `pip install` and no browser to download.

## What is required

| Requirement | Why | Check | If missing |
|---|---|---|---|
| **Google Chrome** (`/Applications/Google Chrome.app`) | `render.sh` launches it headless and clips the plate over CDP | `ls "/Applications/Google Chrome.app"` | Install from google.com/chrome. Chromium works too — repoint `CHROME=` in `render.sh`. |
| **Node ≥ 20** | `shoot.mjs` uses the built-in `fetch` + `WebSocket` (no deps) | `node --version` | Install via nvm (`nvm install 20`) or Homebrew (`brew install node`). |
| **python3** | serves the working dir so Chrome can load the `.dc.html` over `http://` (the Browser MCP cannot open `file://`) | `python3 --version` | Ships with macOS; else `brew install python`. |
| **Network to `unpkg.com`** | runtime-generated plates load React/Babel at render time | first render succeeds | On an offline box, use the static-extract fallback (see `references/pipeline.md`). |

## What is deliberately NOT used

- **No Playwright / Puppeteer.** Playwright's browser binaries may already sit in
  `~/Library/Caches/ms-playwright` from other tools, and the Playwright *library* would
  give a tidier `element.screenshot()` — but installing it is an npm dependency this skill
  chooses to avoid. The Chrome-over-CDP path needs nothing installed.
- **No official Claude Design export.** As of this writing there is no programmatic,
  per-figure image export from Claude Design (the DesignSync MCP is file-CRUD only; UI
  exports are whole-document PDF/PPTX/HTML). Rendering locally is the supported path. If an
  official render surface appears (e.g. a design MCP method), prefer it and keep this as
  fallback — see the non-negotiable in `SKILL.md`.

## Optional: xelatex (only if the consuming document is a LaTeX/pandoc build)

Not needed to produce the PNG. If your project builds with xelatex and it is not on PATH
(common on macOS: `/Library/TeX/texbin`), prepend it: `export PATH="/Library/TeX/texbin:$PATH"`.

## Smoke test

```sh
bash render.sh <workdir> "<Component>.dc.html" /tmp/plate.png 3 '<css-selector>' '' <target-width-mm>
# prints:  PLATE {...}  and  WROTE /tmp/plate.png <WxH> | aspect A:1 | true height at <target-width-mm>mm = N mm
# (omit <target-width-mm> and only the relative aspect is printed)
```
Then open `/tmp/plate.png` to confirm the crop. A `plate not found` error means auto-detect
missed the box: pass an explicit `--selector` (5th arg).
