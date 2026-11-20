// 2026-11-20: Antigravity in the cloud. Its sign-in runs in a cloud machine
// like Claude Code's: measured on agy 1.5.13 in a Debian container through a
// pipe, `agy acp --login` prints a Google link, reads the code as a line, and
// either signs in or ends with `couldn't log in: token exchange failed` and
// exit 1. Signed out, `agy models` exits 1. Where agy keeps the login on a
// machine with no keyring was not seen, so what is kept is every small file the
// sign-in wrote under its home, its logs left out, and a run gets them back
// under its home and under the agy home the Mac names (AGENTBOX_AGY_HOME).
import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import http from 'node:http';
import { execFileSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { createSandbox, pickBaseEnv } from '../cloud/sandbox/server.mjs';
import { AGY_REFRESH } from './fixtures/cloud/fake-vendor-cli.mjs';

const FAKE = fileURLToPath(new URL('./fixtures/cloud/fake-vendor-cli.mjs', import.meta.url));
const git = (cwd: string, ...a: string[]) => execFileSync('git', a, { cwd, encoding: 'utf8' }).trim();

let tmp: string; let files: Map<string, Buffer>; let fileServer: http.Server; let base: string; let bin: string;
let sandbox: ReturnType<typeof createSandbox> | null = null;

beforeEach(async () => {
  tmp = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), 'cloud-agy-')));
  files = new Map();
  fileServer = http.createServer((req, res) => {
    if (req.method === 'GET') {
      const body = files.get(req.url!);
      if (!body) { res.writeHead(404); return res.end(); }
      res.writeHead(200); return res.end(body);
    }
    const chunks: Buffer[] = [];
    req.on('data', (c) => chunks.push(c));
    req.on('end', () => { files.set(req.url!, Buffer.concat(chunks)); res.writeHead(200); res.end(); });
  });
  await new Promise<void>((r) => fileServer.listen(0, '127.0.0.1', r));
  base = `http://127.0.0.1:${(fileServer.address() as { port: number }).port}`;
  bin = path.join(tmp, 'bin');
  fs.mkdirSync(bin);
  fs.writeFileSync(path.join(bin, 'agy'), `#!/bin/sh\nexec "${process.execPath}" "${FAKE}" agy "$@"\n`, { mode: 0o755 });
});

afterEach(async () => {
  await sandbox?.close();
  sandbox = null;
  fileServer.closeAllConnections();
  await new Promise((r) => fileServer.close(r));
  fs.rmSync(tmp, { recursive: true, force: true });
});

const makeSandbox = (mode = '') => {
  sandbox = createSandbox({
    workRoot: path.join(tmp, 'vm-work'), pathRoot: path.join(tmp, 'vm'), bridgeSocket: path.join(tmp, 'bridge.sock'),
    baseEnv: { ...pickBaseEnv(process.env), PATH: `${bin}:${process.env.PATH}`, ...(mode ? { FAKE_VENDOR_MODE: mode } : {}) },
    terminal: (line: string) => ['sh', '-c', line], uploadRetryMs: 10,
  });
  return sandbox;
};

async function until<T>(fn: () => Promise<T>, what: string, ms = 15_000): Promise<T> {
  const end = Date.now() + ms;
  for (;;) {
    const v = await fn();
    if (v) return v;
    if (Date.now() > end) throw new Error(`timed out waiting for ${what}`);
    await new Promise((r) => setTimeout(r, 20));
  }
}

type Event = { kind?: string; state?: string; url?: string; message?: string; email?: string; data?: string; id?: string };
const allEvents = async (): Promise<Event[]> => (await sandbox!.handle({ action: 'events', since: 0, maxBytes: 8 * 1024 * 1024 })).events;
const lastSignin = async () => (await allEvents()).filter((e) => e.kind === 'signin').at(-1);
const ended = () => until(async () => { const s = await lastSignin(); return ['saved', 'failed'].includes(s?.state ?? '') && s; }, 'the end');

describe('signing in to Antigravity in the cloud', () => {
  it('sends the Mac Google’s link, takes the pasted code, and keeps only the small files the sign-in wrote', async () => {
    makeSandbox();
    expect((await sandbox!.handle({ action: 'signin', runId: 'si-a', engine: 'antigravity', saveUrl: `${base}/login-agy` })).ok).toBe(true);
    const asks = await until(async () => (await allEvents()).find((e) => e.kind === 'signin' && e.state === 'needs-code'), 'the link');
    expect(asks!.url).toMatch(/^https:\/\/accounts\.google\.com\/o\/oauth2\/auth\?access_type=offline&client_id=x&/);
    expect((await sandbox!.handle({ action: 'signin-code', runId: 'si-a', code: '  good-agy-code \n' })).ok).toBe(true);
    const end = await ended();
    expect(end, end?.message).toMatchObject({ state: 'saved', email: 'person@example.com' });
    const kept = JSON.parse(files.get('/login-agy')!.toString());
    expect(kept.engine).toBe('antigravity');
    expect(Object.keys(kept.files)).toEqual(['.gemini/antigravity-cli/fake-login.json']);
    expect(Buffer.from(kept.files['.gemini/antigravity-cli/fake-login.json'], 'base64').toString()).toContain(AGY_REFRESH);
    expect(JSON.stringify(await allEvents())).not.toContain(AGY_REFRESH);
  });

  it('says Google did not take a wrong code, and keeps nothing', async () => {
    makeSandbox();
    await sandbox!.handle({ action: 'signin', runId: 'si-b', engine: 'antigravity', saveUrl: `${base}/login-agy` });
    await until(async () => (await allEvents()).find((e) => e.kind === 'signin' && e.state === 'needs-code'), 'the link');
    await sandbox!.handle({ action: 'signin-code', runId: 'si-b', code: 'not-the-code' });
    const end = await ended();
    expect(end!.state).toBe('failed');
    expect(end!.message).toMatch(/Google/);
    expect(files.has('/login-agy')).toBe(false);
  });

  it('keeps nothing when agy itself will not answer on the new sign-in', async () => {
    makeSandbox('agy-refuses');
    await sandbox!.handle({ action: 'signin', runId: 'si-c', engine: 'antigravity', saveUrl: `${base}/login-agy` });
    await until(async () => (await allEvents()).find((e) => e.kind === 'signin' && e.state === 'needs-code'), 'the link');
    await sandbox!.handle({ action: 'signin-code', runId: 'si-c', code: 'good-agy-code' });
    expect((await ended())!.state).toBe('failed');
    expect(files.has('/login-agy')).toBe(false);
  });
});

describe('a run on an Antigravity login kept in the cloud', () => {
  function packedRepo() {
    const repo = path.join(tmp, 'mac', 'repo');
    fs.mkdirSync(repo, { recursive: true });
    git(repo, 'init', '-q', '-b', 'main');
    git(repo, 'config', 'user.email', 'a@example.com');
    git(repo, 'config', 'user.name', 'A');
    fs.writeFileSync(path.join(repo, 'a.txt'), 'one\n');
    git(repo, 'add', '.');
    git(repo, 'commit', '-q', '-m', 'first');
    git(repo, 'checkout', '-q', '-b', 'agentbox/w-1');
    const bundle = path.join(tmp, 'in.bundle');
    git(repo, 'bundle', 'create', bundle, 'agentbox/w-1', 'main');
    files.set('/in.bundle', fs.readFileSync(bundle));
    return repo;
  }

  it('lays the login under the home and under the agy home the Mac names, and never lets the run print it', async () => {
    makeSandbox();
    const login = JSON.stringify({ refresh_token: AGY_REFRESH });
    files.set('/login', Buffer.from(JSON.stringify({ engine: 'antigravity', files: { '.gemini/antigravity-cli/fake-login.json': Buffer.from(login).toString('base64') } })));
    const agyHome = path.join(tmp, 'mac', 'agy-home');
    const placed = path.join(tmp, 'vm', agyHome, 'antigravity-cli', 'fake-login.json');
    const repo = packedRepo();
    const out = await sandbox!.handle({
      action: 'start', runId: 'r-1', cwd: repo, branch: 'agentbox/w-1', bundleUrl: `${base}/in.bundle`,
      argv: [path.join(bin, 'agy'), 'acp'], env: { AGENTBOX_AGY_HOME: agyHome, FAKE_AGY_LOGIN: placed },
      login: { engine: 'antigravity', id: 'L1', url: `${base}/login` },
      results: { bundleUrl: `${base}/out.bundle`, changesUrl: `${base}/changes.bundle`, eventsUrl: `${base}/events.jsonl`, doneUrl: `${base}/done.json`, bring: [] },
    });
    expect(out.ok, JSON.stringify(out)).toBe(true);
    await until(async () => files.has('/done.json') || (await allEvents()).some((e) => e.kind === 'state' && e.state === 'failed'), 'the results');
    expect(fs.readFileSync(placed, 'utf8')).toBe(login);
    const home = path.join(tmp, 'vm', process.env.HOME!, '.gemini', 'antigravity-cli', 'fake-login.json');
    expect(fs.readFileSync(home, 'utf8')).toBe(login);
    const events = await allEvents();
    expect(events.filter((e) => e.kind === 'stdout').map((e) => Buffer.from(e.data!, 'base64').toString()).join('')).toContain('"login"');
    expect(JSON.stringify(events)).not.toContain(AGY_REFRESH);
  });

  it('starts nothing on a login that is not Antigravity’s, and names it missing', async () => {
    makeSandbox();
    files.set('/login', Buffer.from(JSON.stringify({ engine: 'claude', token: 'x' })));
    const repo = packedRepo();
    await sandbox!.handle({
      action: 'start', runId: 'r-2', cwd: repo, branch: 'agentbox/w-1', bundleUrl: `${base}/in.bundle`,
      argv: [path.join(bin, 'agy'), 'acp'], env: { AGENTBOX_AGY_HOME: path.join(tmp, 'mac', 'agy-home') },
      login: { engine: 'antigravity', id: 'L2', url: `${base}/login` },
      results: { bundleUrl: `${base}/out.bundle`, changesUrl: `${base}/changes.bundle`, eventsUrl: `${base}/events.jsonl`, doneUrl: `${base}/done.json`, bring: [] },
    });
    const missing = await until(async () => (await allEvents()).find((e) => e.kind === 'login'), 'the login state');
    expect(missing).toMatchObject({ state: 'missing', id: 'L2' });
  });

  it('refuses an engine it does not run', async () => {
    makeSandbox();
    expect((await sandbox!.handle({ action: 'signin', runId: 'si-x', engine: 'gemini', saveUrl: `${base}/x` })).ok).toBe(false);
  });
});
