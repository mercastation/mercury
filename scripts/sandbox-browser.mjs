// THE PROJECT'S BROWSER GOES INTO THE CLOUD IMAGE AS THE MAC'S OWN CODE.
//
// The image is built from cloud/sandbox alone, so the files a cloud run's copy
// of the browser runs (main/project-browser/cloud-host.mts and what it imports)
// are copied into cloud/sandbox/browser at the same paths, with the packages
// pinned to the versions this app runs. Run this after changing any of them;
// tests/the-cloud-runs-the-macs-own-browser-code fails until you do.
//
//   node scripts/sandbox-browser.mjs
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
export const SANDBOX_BROWSER = path.join(root, 'cloud', 'sandbox', 'browser');

export const BROWSER_FILES = [
  'main/project-browser/cloud-host.mts',
  'main/project-browser/host.mts',
  'main/project-browser/place.mts',
  'main/project-browser/guard.mts',
  'main/project-browser/settle.mts',
  'main/project-browser/sign-in.mts',
  'main/project-browser/view.mts',
  'mcp/project-browser.mts',
  'shared/project-browser.mts',
  'shared/hidden-browser.mts',
  'shared/product-name.mjs',
];

export const BROWSER_PACKAGES = ['@playwright/mcp', 'playwright-core', '@modelcontextprotocol/sdk'];

/** The versions this app's lock file installs. */
export function lockedVersions() {
  const lock = JSON.parse(fs.readFileSync(path.join(root, 'package-lock.json'), 'utf8'));
  return Object.fromEntries(BROWSER_PACKAGES.map((name) => [name, lock.packages[`node_modules/${name}`].version]));
}

export function sandboxPackage() {
  return { name: 'agentbox-sandbox-browser', private: true, description: 'Copied by scripts/sandbox-browser.mjs; do not edit here.', dependencies: lockedVersions() };
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  for (const rel of BROWSER_FILES) {
    const to = path.join(SANDBOX_BROWSER, rel);
    fs.mkdirSync(path.dirname(to), { recursive: true });
    fs.copyFileSync(path.join(root, rel), to);
  }
  const pkg = path.join(SANDBOX_BROWSER, 'package.json');
  const next = `${JSON.stringify(sandboxPackage(), null, 2)}\n`;
  const changed = !fs.existsSync(pkg) || fs.readFileSync(pkg, 'utf8') !== next;
  fs.writeFileSync(pkg, next);
  console.log(`copied ${BROWSER_FILES.length} files into ${path.relative(root, SANDBOX_BROWSER)}`);
  if (changed) console.log('the packages changed: run `npm install --package-lock-only --ignore-scripts` in that folder and commit its package-lock.json');
}
