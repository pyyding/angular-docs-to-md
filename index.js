#!/usr/bin/env node

import fs   from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));

// ── regexes ──────────────────────────────────────────────────────────────────

const HEADER_RE   = /<docs-decorative-header title="([^"]+)"[^>]*>[\s\S]*?<\/docs-decorative-header>/g;
const PILL_ROW_RE = /<docs-pill-row>([\s\S]*?)<\/docs-pill-row>/g;

// ── cache ─────────────────────────────────────────────────────────────────────

const CACHE_DIR = path.join(__dirname, '.cache');
const CACHE_TTL = 24 * 60 * 60 * 1000; // 24 hours

function urlToCacheFile(url) {
  const slug = url.trim()
    .replace(/\/$/, '')
    .replace(/^https?:\/\/angular\.dev\//, '')
    .replace(/[^a-z0-9_-]/gi, '--');
  return path.join(CACHE_DIR, `${slug}.md`);
}

function cacheRead(file) {
  try {
    const stat = fs.statSync(file);
    const age  = Date.now() - stat.mtimeMs;
    if (age > CACHE_TTL) return null;          // stale
    return fs.readFileSync(file, 'utf8');
  } catch {
    return null;                               // missing
  }
}

function cacheWrite(file, content) {
  fs.mkdirSync(path.dirname(file), { recursive: true });
  fs.writeFileSync(file, content, 'utf8');
}

// ── http ──────────────────────────────────────────────────────────────────────

async function fetchText(url) {
  const res = await fetch(url, {
    headers: { 'User-Agent': 'angular-docs-to-md/1.0' },
    signal: AbortSignal.timeout(15_000),
  });
  if (!res.ok) throw new Error(`HTTP ${res.status} fetching ${url}`);
  return res.text();
}

// ── transforms ────────────────────────────────────────────────────────────────

function replaceDecorativeHeaders(md) {
  return md.replace(HEADER_RE, (_, title) => `# ${title}`);
}

function replacePillRows(md) {
  return md.replace(PILL_ROW_RE, (_, inner) => {
    const links = [];
    let m;
    const re = /<docs-pill href="([^"]+)" title="([^"]+)"\/>/g;
    while ((m = re.exec(inner)) !== null) links.push(`- [${m[2]}](${m[1]})`);
    return links.join('\n');
  });
}

async function expandCodeRefs(inner) {
  const codeRe = /<docs-code header="([^"]+)" path="([^"]+)"\/>/g;
  let out = '';
  let m;
  while ((m = codeRe.exec(inner)) !== null) {
    const header  = m[1];
    const path    = m[2];
    const rawUrl  = `https://raw.githubusercontent.com/angular/angular/main/${path}`;
    const content = await fetchText(rawUrl);
    const ext     = header.split('.').pop() ?? '';
    out += `\`\`\`${ext}\n// ${header}\n${content}\n\`\`\`\n\n`;
  }
  return out.trimEnd();
}

async function expandTabGroups(md, examplesPerGroup) {
  const replacements = [];
  const groupRe = /<docs-tab-group>([\s\S]+?)<\/docs-tab-group>/g;
  let gm;
  while ((gm = groupRe.exec(md)) !== null) {
    const tabRe = /<docs-tab label="([^"]+)">([\s\S]+?)<\/docs-tab>/g;
    let groupMd = '';
    let tm;
    let count = 0;
    while ((tm = tabRe.exec(gm[1])) !== null && count < examplesPerGroup) {
      groupMd += `**Example: ${tm[1]}**\n\n`;
      groupMd += await expandCodeRefs(tm[2]);
      groupMd += '\n\n';
      count++;
    }
    replacements.push({ start: gm.index, end: gm.index + gm[0].length, rep: groupMd.trimEnd() });
  }
  return applyReplacements(md, replacements);
}

async function expandMultifileBlocks(md) {
  const replacements = [];
  const re = /<docs-code-multifile[^>]*>([\s\S]*?)<\/docs-code-multifile>/g;
  let m;
  while ((m = re.exec(md)) !== null) {
    const rep = await expandCodeRefs(m[1]);
    replacements.push({ start: m.index, end: m.index + m[0].length, rep });
  }
  return applyReplacements(md, replacements);
}

function applyReplacements(str, replacements) {
  for (const { start, end, rep } of [...replacements].reverse()) {
    str = str.slice(0, start) + rep + str.slice(end);
  }
  return str;
}

// ── public ────────────────────────────────────────────────────────────────────

async function convertAngularDocs(url, {
  examplesPerGroup = 1,
  parseHeader = true,
  parsePills  = true,
  refresh     = false,
} = {}) {
  url = url.trim().replace(/\/$/, '');

  const cacheFile = urlToCacheFile(url);

  if (!refresh) {
    const cached = cacheRead(cacheFile);
    if (cached !== null) {
      process.stderr.write(`[cache] hit  ${cacheFile}\n`);
      return cached;
    }
    try {
      fs.statSync(cacheFile);
      // file exists but was stale (cacheRead returned null)
      process.stderr.write(`[cache] stale ${cacheFile} — re-fetching\n`);
    } catch {
      // file doesn't exist yet — first fetch, say nothing
    }
  } else {
    process.stderr.write(`[cache] refresh requested — re-fetching\n`);
  }

  const docPath = url.replace(/^https?:\/\/angular\.dev\//, '');
  const rawUrl  = `https://raw.githubusercontent.com/angular/angular/main/adev/src/content/${docPath}.md`;

  let body = await fetchText(rawUrl);
  if (parseHeader) body = replaceDecorativeHeaders(body);
  if (parsePills)  body = replacePillRows(body);
  body = await expandTabGroups(body, examplesPerGroup);
  body = await expandMultifileBlocks(body);

  cacheWrite(cacheFile, body);
  process.stderr.write(`[cache] saved ${cacheFile}\n`);

  return body;
}

// ── CLI ───────────────────────────────────────────────────────────────────────

const args = process.argv.slice(2);

if (args.length === 0 || args.includes('--help') || args.includes('-h')) {
  console.error('Usage: angular-docs-to-md <URL> [options]');
  console.error('');
  console.error('Options:');
  console.error('  --examples N    tab examples to expand per group (default: 1)');
  console.error('  --no-header     skip <docs-decorative-header> parsing');
  console.error('  --no-pills      skip <docs-pill-row> parsing');
  console.error('  --refresh       ignore cache and re-fetch (overwrites cached file)');
  console.error('  --no-cache      skip both reading and writing the cache');
  console.error('');
  console.error(`Cache directory: ${CACHE_DIR}`);
  process.exit(args.length === 0 ? 1 : 0);
}

const url = args[0];
if (!url.includes('angular.dev')) {
  console.error('error: URL must be from angular.dev');
  process.exit(1);
}

const examplesIdx      = args.indexOf('--examples');
const examplesPerGroup = examplesIdx !== -1 ? (parseInt(args[examplesIdx + 1], 10) || 1) : 1;
const parseHeader      = !args.includes('--no-header');
const parsePills       = !args.includes('--no-pills');
const refresh          = args.includes('--refresh');
const noCache          = args.includes('--no-cache');

async function main() {
  if (noCache) {
    // Bypass cache entirely — fetch and print, don't save
    const url2     = args[0].trim().replace(/\/$/, '');
    const docPath  = url2.replace(/^https?:\/\/angular\.dev\//, '');
    const rawUrl   = `https://raw.githubusercontent.com/angular/angular/main/adev/src/content/${docPath}.md`;
    let body = await fetchText(rawUrl);
    if (parseHeader) body = replaceDecorativeHeaders(body);
    if (parsePills)  body = replacePillRows(body);
    body = await expandTabGroups(body, examplesPerGroup);
    body = await expandMultifileBlocks(body);
    return body;
  }
  return convertAngularDocs(url, { examplesPerGroup, parseHeader, parsePills, refresh });
}

main()
  .then(md => process.stdout.write(md + '\n'))
  .catch(err => { console.error(`error: ${err.message}`); process.exit(1); });
