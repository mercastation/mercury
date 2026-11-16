// A CLOUD SANDBOX SENDS HOME WHAT ITS PROJECT'S BROWSER RENEWED.
//
// A cloud run's copy of its project's browser (main/project-browser/cloud-host.mts)
// writes the cookies its sites renewed to the file the Mac named at the start
// (`browser.out`), and the sandbox uploads it with the run's results, so the
// Mac's profile keeps the newer sign-in (2026-11-15). The server runs as root,
// so it sends only a plain file there: a link in its place would have it send
// whatever the link points at.
import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import http from 'node:http';
import { execFileSync } from 'node:child_process';
import { createSandbox } from '../cloud/sandbox/server.mjs';

const git = (cwd: string, ...args: string[]) => execFileSync('git', ['-c', 'gc.auto=0', '-c', 'maintenance.auto=false', ...args], { cwd, encoding: 'utf8' }).trim();
const OUT = '/tmp/mercury-browser-run-b-renewed.json';

let tmp: string; let files: Map<string, Buffer>; let server: http.Server; let base: string;
let sandbox: ReturnType<typeof createSandbox> | null = null;

beforeEach(async () => {
  tmp = fs.mkdtempSync(path.join(os.tmpdir(), 'sandbox-browser-'));
  const repo = path.join(tmp, 'mac', 'repo');
  fs.mkdirSync(repo, { recursive: true });
  git(repo, 'init', '-q', '-b', 'main');
  git(repo, '-c', 'user.email=a@example.com', '-c', 'user.name=A', 'commit', '-q', '--allow-empty', '-m', 'first');
  git(repo, 'checkout', '-q', '-b', 'agentbox/w-1');
  git(repo, 'bundle', 'create', path.join(tmp, 'in.bundle'), 'agentbox/w-1', 'main');
  files = new Map([['/in.bundle', fs.readFileSync(path.join(tmp, 'in.bundle'))]]);
  server = http.createServer((req, res) => {
    if (req.method === 'GET') { const body = files.get(req.url!); res.writeHead(body ? 200 : 404); res.end(body); return; }
    const chunks: Buffer[] = [];
    req.on('data', (c) => chunks.push(c));
    req.on('end', () => { files.set(req.url!, Buffer.concat(chunks)); res.writeHead(200); res.end(); });
  });
  await new Promise<void>((r) => server.listen(0, '127.0.0.1', r));
  base = `http://127.0.0.1:${(server.address() as { port: number }).port}`;
});

afterEach(async () => {
  await sandbox?.close();
  sandbox = null;
  server.closeAllConnections();
  await new Promise((r) => server.close(r));
  try { fs.rmSync(tmp, { recursive: true, force: true, maxRetries: 10, retryDelay: 100 }); } catch { /* best effort */ }
});

async function until(fn: () => boolean, what: string, ms = 20_000): Promise<void> {
  const end = Date.now() + ms;
  while (!fn()) {
    if (Date.now() > end) throw new Error(`timed out waiting for ${what}`);
    await new Promise((r) => setTimeout(r, 25));
  }
}

/** A run whose command does `js` with the file the browser writes named `out`, under the sandbox's path root. */
async function run(js: string, { browser = { out: OUT } as { out: string } | null } = {}) {
  const pathRoot = path.join(tmp, 'vm');
  sandbox = createSandbox({ workRoot: path.join(tmp, 'work'), pathRoot });
  const started = await sandbox.handle({
    action: 'start', runId: 'run-b', cwd: '/w', branch: 'agentbox/w-1', bundleUrl: `${base}/in.bundle`,
    argv: [process.execPath, '-e', `const fs=require("fs"),path=require("path");const out=path.join(process.env.AGENTBOX_PATH_ROOT,${JSON.stringify(OUT)});fs.mkdirSync(path.dirname(out),{recursive:true});${js}`],
    env: {},
    ...(browser ? { browser } : {}),
    results: { bundleUrl: `${base}/r/out.bundle`, changesUrl: `${base}/r/changes.bundle`, eventsUrl: `${base}/r/events.jsonl`, doneUrl: `${base}/r/done.json`, browserUrl: `${base}/r/browser.json`, bring: [] },
  });
  await until(() => files.has('/r/done.json'), 'the run\'s results');
  return started;
}

describe('a cloud run with its project\'s browser', () => {
  it('says the image can run one', async () => {
    const started = await run('');
    expect(started.caps).toContain('browser');
  });

  it('sends the renewed cookies home with its results', async () => {
    const renewed = JSON.stringify({ cookies: [{ name: 'session', value: 'renewed', domain: 'gitlab.example.gov', path: '/', expires: 1_900_000_000 }] });
    await run(`fs.writeFileSync(out, ${JSON.stringify(renewed)});`);
    expect(files.get('/r/browser.json')?.toString()).toBe(renewed);
  });

  it('sends nothing when the browser renewed nothing', async () => {
    await run('');
    expect(files.has('/r/browser.json')).toBe(false);
  });

  it('sends nothing a link stands in for', async () => {
    const secret = path.join(tmp, 'secret.json');
    fs.writeFileSync(secret, '{"login":"not for the Mac"}');
    await run(`fs.symlinkSync(${JSON.stringify(secret)}, out);`);
    expect(files.has('/r/browser.json')).toBe(false);
  });

  it('sends nothing that is not JSON, and nothing for a start that named no browser', async () => {
    await run('fs.writeFileSync(out, "not json");');
    expect(files.has('/r/browser.json')).toBe(false);
    await sandbox!.close();
    files.delete('/r/done.json');
    await run('fs.writeFileSync(out, "{}");', { browser: null });
    expect(files.has('/r/browser.json')).toBe(false);
  });

  it('sends nothing from outside the sandbox\'s /tmp', async () => {
    await run(`fs.writeFileSync(out, "{}");`, { browser: { out: '/w/../etc/renewed.json' } });
    expect(files.has('/r/browser.json')).toBe(false);
  });
});
