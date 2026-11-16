// THE CLOUD RUNS THE MAC'S OWN BROWSER CODE.
//
// A cloud run's copy of its project's browser is the Mac's host
// (main/project-browser/cloud-host.mts and what it imports), copied into the
// sandbox image because the image is built from cloud/sandbox alone
// (2026-11-15). A copy that drifts would open other sites, offer other tools or
// drop the page guard in the cloud only, where nobody is watching, so each file
// must be the Mac's byte for byte, on the packages this app runs. After
// changing one, run `node scripts/sandbox-browser.mjs`.
import { describe, expect, it } from 'vitest';
import fs from 'node:fs';
import path from 'node:path';
import { BROWSER_FILES, BROWSER_PACKAGES, SANDBOX_BROWSER, lockedVersions, sandboxPackage } from '../scripts/sandbox-browser.mjs';

const ROOT = path.resolve(import.meta.dirname, '..');
const read = (p: string) => fs.readFileSync(p, 'utf8');

describe('the browser code in the cloud image', () => {
  it.each(BROWSER_FILES)('%s is the Mac\'s own, as it is now', (rel) => {
    expect(read(path.join(SANDBOX_BROWSER, rel)), `run node scripts/sandbox-browser.mjs: ${rel} changed`).toBe(read(path.join(ROOT, rel)));
  });

  it('holds every file the cloud host imports, and nothing else of the app', () => {
    const relative = (from: string) => [...read(path.join(ROOT, from)).matchAll(/^import [^;]*? from '(\.[^']+)';/gm)].map((m) => path.relative(ROOT, path.resolve(path.dirname(path.join(ROOT, from)), m[1])));
    const reached = new Set<string>();
    const walk = (rel: string) => { if (reached.has(rel)) return; reached.add(rel); for (const next of relative(rel)) walk(next); };
    walk('main/project-browser/cloud-host.mts');
    expect([...reached].sort()).toEqual([...BROWSER_FILES].sort());
  });

  it('runs on the packages this app runs, locked', () => {
    expect(JSON.parse(read(path.join(SANDBOX_BROWSER, 'package.json')))).toEqual(sandboxPackage());
    const lock = JSON.parse(read(path.join(SANDBOX_BROWSER, 'package-lock.json')));
    for (const [name, version] of Object.entries(lockedVersions())) expect(lock.packages[`node_modules/${name}`]?.version, name).toBe(version);
    expect(Object.keys(lockedVersions())).toEqual(BROWSER_PACKAGES);
  });

  it('is installed in the image where the Mac starts it', () => {
    const docker = read(path.join(ROOT, 'cloud', 'sandbox', 'Dockerfile'));
    expect(docker).toMatch(/^COPY browser \/opt\/agentbox\/browser$/m);
    expect(docker).toMatch(/npm ci --omit=dev --ignore-scripts/);
    expect(read(path.join(ROOT, 'main', 'cloud', 'browser.mts'))).toContain("'/opt/agentbox/browser/main/project-browser/cloud-host.mts'");
  });
});
