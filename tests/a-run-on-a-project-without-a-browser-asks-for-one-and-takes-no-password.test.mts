// A thread on a project with no sites listed was told nothing about browsers,
// so when its work needed a signed-in GitLab it read the site's password out of
// git's stored credentials (`git credential fill`) and typed it into a
// Playwright it installed in /tmp. Seen in the thread's own log on 2026-11-15;
// the owner had never been told the project could have a browser of its own.
// A run now hears, with its connected tools, how the owner gives the project a
// browser, and what to say when its browser's tools are not in this run.

import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { describe, expect, it } from 'vitest';
import { Supervisor } from '../main/supervisor.mts';
import { browserBrief, NO_BROWSER_YET } from '../shared/project-browser.mts';

const REPO = path.resolve(import.meta.dirname, '..');
const tmp = () => fs.mkdtempSync(path.join(os.tmpdir(), 'no-browser-brief-'));

const systemOf = (product: string, { connections = true } = {}) => {
  const dir = tmp();
  const store = { listItems: () => [], listProducts: () => [], isDue: () => true };
  const sup = new Supervisor({ storeRoot: dir, maxConcurrentSessions: 1, projectBrowserSites: { oms: ['acme.atlassian.net'] } }, store, REPO, REPO, dir);
  if (connections) sup.connections = { catalog: [], workingNow: () => [], statusNow: () => [] };
  const row = { id: 'w-1a2b3c4d5e', product, title: 'Read the board', body: 'Do it.', labels: ['founder'] };
  return sup.spawnPlan(row, { slug: product, dir: os.tmpdir(), repoPath: null }, { engine: 'claude' }).system as string;
};

describe('a run on a project with no browser', () => {
  it('is told, with its connected tools, where the owner gives the project one', () => {
    const system = systemOf('notes');
    expect(system).toContain(NO_BROWSER_YET);
    expect(system.indexOf(NO_BROWSER_YET)).toBeGreaterThan(system.indexOf('## Connected tools'));
    expect(NO_BROWSER_YET).toContain('Settings, Projects, this project, Browser');
  });

  it('is told not to drive a browser of its own or use a password it finds', () => {
    expect(NO_BROWSER_YET).toMatch(/do not drive a browser of your own/i);
    expect(NO_BROWSER_YET).toMatch(/git's stored credentials/);
    expect(NO_BROWSER_YET).toMatch(/keychain/);
  });

  it('is told nothing of it where the app has no connected tools to tell it with', () => {
    expect(systemOf('notes', { connections: false }) ?? '').not.toContain(NO_BROWSER_YET);
  });
});

describe('a run on a project with a browser', () => {
  it('is not told the project has none', () => {
    const system = systemOf('oms');
    expect(system).not.toContain(NO_BROWSER_YET);
    expect(system).toContain('acme.atlassian.net');
  });

  it('is told what to say when the browser\'s tools are not in this run', () => {
    expect(browserBrief(['acme.atlassian.net'])).toMatch(/If no project-browser tools are in your tool list/);
  });

  it('names only its own project\'s sites to a run on another project', () => {
    expect(systemOf('notes')).not.toContain('acme.atlassian.net');
  });
});
