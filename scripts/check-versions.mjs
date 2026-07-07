// Release-time version validation.
//
// Fails when the version in package.json, Cargo.toml, tauri.conf.json,
// and the git tag (when on a tagged commit) do not match.
//
// Usage:
//   node scripts/check-versions.mjs            # check internal consistency
//   node scripts/check-versions.mjs --tag v0.1.1   # also check tag
//
// Exits non-zero on mismatch. Intended to run in CI before building.

import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, join } from 'node:path';

const root = join(dirname(fileURLToPath(import.meta.url)), '..');

function readJson(path) {
  return JSON.parse(readFileSync(join(root, path), 'utf8'));
}

function readTomlVersion(path) {
  const text = readFileSync(join(root, path), 'utf8');
  const m = text.match(/^version\s*=\s*"([^"]+)"/m);
  if (!m) {
    throw new Error(`could not find [package] version in ${path}`);
  }
  return m[1];
}

const checks = [
  ['package.json', readJson('package.json').version],
  ['src-tauri/Cargo.toml', readTomlVersion('src-tauri/Cargo.toml')],
  ['src-tauri/tauri.conf.json', readJson('src-tauri/tauri.conf.json').version],
];

const tagArg = process.argv.find((a) => a.startsWith('--tag='));
const tagValue = tagArg ? tagArg.slice('--tag='.length) : null;

let ok = true;

const versions = checks.map(([file, v]) => {
  console.log(`${file}: ${v}`);
  return v;
});

const first = versions[0];
for (const [file, v] of checks) {
  if (v !== first) {
    console.error(`MISMATCH: ${file} is ${v}, expected ${first}`);
    ok = false;
  }
}

if (tagValue !== null) {
  const expected = tagValue.replace(/^v/, '');
  console.log(`git tag: ${tagValue} -> expected version ${expected}`);
  if (expected !== first) {
    console.error(`MISMATCH: tag ${tagValue} does not match version ${first}`);
    ok = false;
  }
}

if (ok) {
  console.log('OK: all versions aligned.');
  process.exit(0);
} else {
  process.exit(1);
}
