// A PROJECT'S BROWSER IN A CLOUD RUN.
//
// The run's machine has no profile of its own: the Mac sends the sign-ins its
// project browser holds (host.mts `state`), and this serves them to the one run
// here with the same host and the same tools as on the Mac, on the machine's
// Chromium. The cookies the sites renew are written to PROJECT_BROWSER_OUT
// after every call, which the sandbox sends home with the run's results, and
// the Mac takes back only the ones its profile already holds (`cookiesToTake`).
//
// Copied into the sandbox image with the files it imports (cloud/sandbox/browser,
// tests/the-cloud-runs-the-macs-own-browser-code), and started by Claude Code
// there as the run's `project-browser` server (main/cloud/run.mts).
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { chromium } from 'playwright-core';
import { cleanSites } from '../../shared/project-browser.mts';
import { serveTools } from '../../mcp/project-browser.mts';
import { renewedCookies, serveBrowser, type Cookie, type Launch, type StorageState } from './host.mts';
import { writeProject } from './place.mts';

export const CLOUD_CHROME = '/opt/agentbox/chromium.sh';

/** The sign-ins the Mac sent, or none when the file is missing or not one. */
export function readState(file: string | undefined): StorageState {
  try {
    const kept = JSON.parse(fs.readFileSync(file!, 'utf8'));
    return { cookies: Array.isArray(kept?.cookies) ? kept.cookies : [], origins: Array.isArray(kept?.origins) ? kept.origins : [] };
  } catch { return { cookies: [], origins: [] }; }
}

/** A fresh browser holding the sign-ins it was sent, closed with its one context. */
export const launchWith = (given: StorageState): Launch => async (_profile, chrome) => {
  const browser = await chromium.launch({ executablePath: chrome, headless: true, args: ['--disable-blink-features=AutomationControlled'] });
  try {
    const context = await browser.newContext({ storageState: given, viewport: { width: 1280, height: 720 } });
    context.once('close', () => { browser.close().catch(() => {}); });
    return context;
  } catch (err) { await browser.close().catch(() => {}); throw err; }
};

/** Writes what the sites renewed, whole, where the sandbox picks it up. */
export const keepRenewed = (given: readonly Cookie[], out: string) => async (context: { cookies: () => Promise<Cookie[]> }): Promise<void> => {
  const cookies = renewedCookies(given, await context.cookies());
  fs.mkdirSync(path.dirname(out), { recursive: true });
  fs.writeFileSync(`${out}.${process.pid}`, JSON.stringify({ cookies }), { mode: 0o600 });
  fs.renameSync(`${out}.${process.pid}`, out);
};

function main(): void {
  const env = process.env;
  const sites = cleanSites(env.PROJECT_BROWSER_SITES ?? '');
  const given = readState(env.PROJECT_BROWSER_STATE);
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'project-browser-'));
  const pages = path.join(dir, 'pages');
  fs.mkdirSync(pages, { recursive: true });
  // The page files Playwright MCP writes are named against the folder it runs in.
  process.chdir(pages);
  writeProject(dir, { slug: 'cloud', sites, pages });
  const chrome = env.PROJECT_BROWSER_CHROME || CLOUD_CHROME;
  const host = serveBrowser({
    dir,
    sock: path.join(dir, 'host.sock'),
    appPid: process.pid,
    chrome: () => (fs.existsSync(chrome) ? chrome : null),
    launch: launchWith(given),
    ...(env.PROJECT_BROWSER_OUT ? { keep: keepRenewed(given.cookies, env.PROJECT_BROWSER_OUT) } : {}),
    log: (line) => process.stderr.write(`project browser: ${line}\n`),
  });
  const end = () => { host.close().catch(() => {}).finally(() => process.exit(0)); };
  process.on('SIGTERM', end);
  process.on('SIGINT', end);
  host.started.catch((err: unknown) => process.stderr.write(`project browser: ${(err as Error)?.message ?? err}\n`));
  serveTools(async <T,>(request: Parameters<typeof host.answer>[0]) => (await host.answer(request)) as T, env.PROJECT_BROWSER_THREAD || 'cloud');
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) main();
