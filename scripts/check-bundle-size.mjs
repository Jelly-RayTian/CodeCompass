import { readFileSync, statSync } from 'node:fs';
import { resolve } from 'node:path';

const manifestPath = resolve('dist/.vite/manifest.json');
const manifest = JSON.parse(readFileSync(manifestPath, 'utf8'));

const entry = Object.values(manifest).find((chunk) => chunk.isEntry === true);
const viewer = Object.values(manifest).find(
  (chunk) => chunk.name === 'Viewer' && chunk.isDynamicEntry === true,
);

if (entry === undefined) {
  throw new Error(`Entry chunk was not found in ${manifestPath}`);
}

if (viewer === undefined) {
  throw new Error(
    'Viewer is not a dynamic entry. Monaco may have moved back into the initial bundle.',
  );
}

const budgets = [
  { label: 'initial entry', file: entry.file, maxKiB: 500 },
  { label: 'lazy Viewer', file: viewer.file, maxKiB: 4000 },
];

let failed = false;

for (const budget of budgets) {
  const bytes = statSync(resolve('dist', budget.file)).size;
  const kibibytes = bytes / 1024;
  const status = kibibytes <= budget.maxKiB ? 'OK' : 'OVER BUDGET';
  console.log(
    `${status}: ${budget.label} ${kibibytes.toFixed(1)} KiB / ${budget.maxKiB} KiB`,
  );
  failed ||= kibibytes > budget.maxKiB;
}

if (failed) {
  process.exitCode = 1;
}
