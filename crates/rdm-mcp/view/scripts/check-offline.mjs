// Fails if dist/index.html is not self-contained.
// Usage: node scripts/check-offline.mjs [--size]
import { readFileSync, existsSync } from 'node:fs';
import { fileURLToPath } from 'node:url';

const file = fileURLToPath(new URL('../dist/index.html', import.meta.url));
if (!existsSync(file)) {
  console.error('check:offline FAIL: dist/index.html is missing, run the build first');
  process.exit(1);
}
const html = readFileSync(file, 'utf8');
console.log(`dist/index.html: ${Buffer.byteLength(html)} bytes`);
if (process.argv.includes('--size')) process.exit(0);

// Each allowed URL string is exact and justified. Anything else fails.
const ALLOWED_URLS = new Map([
  // Inert identifiers inside bundled zod / JSON-schema code (@modelcontextprotocol/core). None is ever fetched.
  ['https://json-schema.org/draft/2019-09/schema', 'JSON Schema $schema identifier'],
  ['http://json-schema.org/draft/2019-09/schema', 'JSON Schema $schema identifier'],
  ['https://json-schema.org/draft/2020-12/schema', 'JSON Schema $schema identifier'],
  ['http://json-schema.org/draft-07/schema#', 'JSON Schema $schema identifier'],
  ['http://json-schema.org/draft-04/schema#', 'JSON Schema $schema identifier'],
  ['http://[${e}]', 'zod hostname-validation template literal, used to parse an IPv6 host, not to fetch'],
  ['https://github.com/cfworker', 'attribution string in the bundled cfworker JSON-schema validator'],
  ['https://*.example.com', 'zod doc/example string'],
]);

const problems = [];
if (/<script\b[^>]*\bsrc\s*=/i.test(html)) problems.push('<script src>');
if (/<link\b[^>]*\bhref\s*=/i.test(html)) problems.push('<link href>');
if (/\bimport\s*\(/.test(html)) problems.push('import(');
for (const m of new Set(html.match(/https?:\/\/[^\s"'`)<>\\]*/g) ?? [])) {
  if (!ALLOWED_URLS.has(m)) problems.push(`URL string: ${m}`);
}
if (problems.length) {
  console.error('check:offline FAIL:\n  ' + problems.join('\n  '));
  process.exit(1);
}
console.log(`check:offline PASS (${ALLOWED_URLS.size} allowed URL strings)`);
