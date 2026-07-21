// decode-dc.mjs — turn a DesignSync get_file result into the real file on disk.
//
// DesignSync's get_file returns JSON like {"method":"get_file","content":"<!DOCTYPE…"}
// where `content` is JSON-escaped (\n, \"). Hand-copying that (especially a component's
// generated renderVals JS) is where transcription errors creep in. This decoder is
// self-verifying: JSON.parse throws loudly on any bad copy, so a file that decodes at
// all decoded correctly.
//
// Two ways the get_file result reaches you:
//   1. Large results (e.g. support.js ~65 KB) auto-persist to a tool-results .txt —
//      pass that path directly.
//   2. Inline results — paste the whole JSON object into a file with a quoted heredoc
//      (quotes stop the shell mangling it), then pass that file:
//        cat > got.json <<'JSON'
//        {"method":"get_file","path":"…","content":"…"}
//        JSON
//
// Usage: node decode-dc.mjs <getfile-json-path> <out-path>
import { readFileSync, writeFileSync } from 'node:fs';
const [src, out] = process.argv.slice(2);
if (!src || !out) { console.error('usage: node decode-dc.mjs <getfile-json> <out>'); process.exit(2); }
const j = JSON.parse(readFileSync(src, 'utf8'));
if (typeof j.content !== 'string') { console.error('no string `content` field in', src); process.exit(3); }
writeFileSync(out, j.content);
console.log('WROTE', out, j.content.length, 'bytes', j.truncated ? '(WARNING: source truncated!)' : '');
