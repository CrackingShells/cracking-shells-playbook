// shoot.mjs — clip a Claude Design ".dc.html" figure plate to a print-quality PNG
// via Chrome over the DevTools Protocol. Zero npm deps (uses Node's built-in fetch
// + WebSocket, Node >= 20). Chrome must already be listening on 127.0.0.1:9222;
// scripts/render.sh launches it, serves the working dir, and calls this for you.
//
// Handles BOTH kinds of asset with one code path:
//   - static plates (hardcoded HTML/SVG)
//   - runtime-generated plates whose artwork is produced by the DC React runtime
//     (support.js + React/Babel from unpkg, e.g. procedural <sc-for> fields)
// Readiness is detected by waiting until the plate's descendant count goes STABLE,
// which covers the runtime case (count grows, then settles) and the static case
// (already settled) without needing to know which one you have.
//
// Usage:
//   node shoot.mjs --url <http url> --out <png> [--scale 3] [--selector <css>] [--field <css-color>]
//
// Plate detection, in priority order:
//   1. --selector <css>   : use exactly this element (most reliable; see references/pipeline.md).
//   2. --field <css-color>: match the largest bordered block painted this colour.
//   3. auto: read the design system's --field token from :root and match the largest
//      bordered block painted that colour, with two known ECL-era colours as fallback.
// The plate is deliberately the figure box only, which EXCLUDES the component's own
// <h2>, editor chrome, and duplicate caption (they live outside that box), so the
// consuming document's own caption stays authoritative.

const args = Object.fromEntries(
  process.argv.slice(2).join(' ').split('--').filter(Boolean)
    .map(s => s.trim()).map(s => { const i = s.indexOf(' '); return i < 0 ? [s, true] : [s.slice(0, i), s.slice(i + 1).trim()]; })
);
const url = args.url;
const out = args.out;
const scale = Number(args.scale || 3);
const selector = args.selector || null;
const field = args.field || null;
if (!url || !out) { console.error('need --url and --out'); process.exit(2); }

// Detect the plate (or use the given selector) and report its rect + child count.
// The plate is the LARGEST element painted the field colour with a visible border;
// inner field-coloured chips are smaller, so max-area wins the outermost exhibit box.
// `norm()` resolves any CSS colour (hex, var value, name) to the canonical rgb()
// string getComputedStyle returns, so token-vs-literal comparisons line up. The
// field colour comes from --field on :root, so this adapts to any project's token
// instead of hardcoding one design system's value.
const PROBE = (sel, fieldColor) => `(() => {
  const norm = (c) => { const d = document.createElement('div'); d.style.color = c; document.body.appendChild(d); const v = getComputedStyle(d).color; d.remove(); return v; };
  let FIELDS = [];
  ${fieldColor
    ? `FIELDS = [norm(${JSON.stringify(fieldColor)})];`
    : `const tok = getComputedStyle(document.documentElement).getPropertyValue('--field').trim();
       if (tok) { try { FIELDS.push(norm(tok)); } catch (e) {} }
       FIELDS.push('rgb(252, 251, 247)', 'rgb(232, 228, 218)');`}
  let el = ${sel ? `document.querySelector(${JSON.stringify(sel)})` : 'null'};
  if (!el) {
    let best = null, bestArea = 0;
    for (const n of document.querySelectorAll('div')) {
      const cs = getComputedStyle(n);
      if (!FIELDS.includes(cs.backgroundColor)) continue;
      if (parseFloat(cs.borderTopWidth) === 0 && parseFloat(cs.borderLeftWidth) === 0) continue;
      const r = n.getBoundingClientRect();
      const area = r.width * r.height;
      if (area > bestArea) { best = n; bestArea = area; }
    }
    el = best;
  }
  if (!el) return { found: false, fields: FIELDS };
  const r = el.getBoundingClientRect();
  return { found: true, x: r.x, y: r.y, w: r.width, h: r.height,
           kids: el.querySelectorAll('*').length,
           pal: document.fonts.check('27px Palatino'),
           hel: document.fonts.check('22px "Helvetica Neue"') };
})()`;

const listing = await (await fetch('http://127.0.0.1:9222/json')).json();
const page = listing.find(t => t.type === 'page');
const ws = new WebSocket(page.webSocketDebuggerUrl);
await new Promise(r => ws.addEventListener('open', r, { once: true }));

let id = 0; const pending = new Map(); const logs = [];
ws.addEventListener('message', ev => {
  const m = JSON.parse(ev.data);
  if (m.id && pending.has(m.id)) { pending.get(m.id)(m); pending.delete(m.id); }
  if (m.method === 'Runtime.consoleAPICalled' && m.params.type === 'error')
    logs.push(m.params.args.map(a => a.value ?? a.description ?? a.type).join(' '));
});
const send = (method, params = {}) => new Promise(res => { const i = ++id; pending.set(i, m => res(m.result)); ws.send(JSON.stringify({ id: i, method, params })); });
const evalp = async e => (await send('Runtime.evaluate', { expression: e, returnByValue: true, awaitPromise: true })).result?.value;

await send('Page.enable'); await send('Runtime.enable');
const loaded = new Promise(r => { const h = ev => { if (JSON.parse(ev.data).method === 'Page.loadEventFired') { ws.removeEventListener('message', h); r(); } }; ws.addEventListener('message', h); });
await send('Page.navigate', { url });
await loaded;

// wait until the plate exists and its subtree stops growing (2 stable reads)
const deadline = Date.now() + 30000;
let probe, prevKids = -1, stable = 0;
while (Date.now() < deadline) {
  probe = await evalp(PROBE(selector, field));
  if (probe?.found && probe.w > 0) {
    if (probe.kids === prevKids) { if (++stable >= 2) break; } else stable = 0;
    prevKids = probe.kids;
  }
  await new Promise(r => setTimeout(r, 250));
}
await evalp('document.fonts.ready.then(()=>true)');
await new Promise(r => setTimeout(r, 500));
probe = await evalp(PROBE(selector, field));

console.log('PLATE', JSON.stringify(probe));
if (logs.length) console.log('CONSOLE_ERRORS(' + logs.length + '): ' + [...new Set(logs)].slice(0, 6).join(' | '));
if (!probe?.found || !probe.w) { console.error('plate not found — pass --selector explicitly (see references/pipeline.md)'); ws.close(); process.exit(3); }
if (!probe.pal || !probe.hel) console.log('WARN: Palatino/Helvetica Neue not both resolved — fonts may be substituted');

const shot = await send('Page.captureScreenshot', {
  format: 'png',
  clip: { x: probe.x, y: probe.y, width: probe.w, height: probe.h, scale },
  captureBeyondViewport: true,
});
const { writeFileSync } = await import('node:fs');
writeFileSync(out, Buffer.from(shot.data, 'base64'));
console.log('WROTE', out, Math.round(probe.w * scale) + 'x' + Math.round(probe.h * scale),
  '| aspect', (probe.w / probe.h).toFixed(3) + ':1',
  '| true height at 168mm =', (168 * probe.h / probe.w).toFixed(1) + 'mm');
ws.close();
process.exit(0);
