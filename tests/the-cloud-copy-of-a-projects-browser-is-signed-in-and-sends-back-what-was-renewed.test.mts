// THE CLOUD COPY OF A PROJECT'S BROWSER IS SIGNED IN, AND SENDS BACK WHAT WAS RENEWED.
//
// Why this exists (2026-11-15): a cloud run now carries its project's browser
// (main/cloud/browser.mts). This drives the real pieces end to end on a real
// Chrome for Testing: the Mac's host is signed in to a site and hands its
// sign-ins over (`state`), the copy of the code that goes into the cloud image
// (cloud/sandbox/browser) serves them as Claude Code would start it there, on
// stdin and stdout, opens only the project's sites, and writes the cookie the
// site renewed; the host here then takes that one back and nothing it did not
// already hold. One local server under two names stands for the sites:
// `localhost` is on the list and `127.0.0.1` is not.
//
// Skipped on a machine with no Chrome for Testing.
import { describe, it, expect, beforeAll, afterAll } from 'vitest';
import fs from 'node:fs';
import http from 'node:http';
import os from 'node:os';
import path from 'node:path';
import readline from 'node:readline';
import { spawn, type ChildProcessWithoutNullStreams } from 'node:child_process';
import type { AddressInfo } from 'node:net';
import type { CallToolResult, Tool } from '@modelcontextprotocol/sdk/types.js';
import { serveBrowser, testBrowser, type Cookie, type StorageState } from '../main/project-browser/host.mts';
import { askHost, writeProject } from '../main/project-browser/place.mts';

const chrome = testBrowser();
const COPY = path.resolve(import.meta.dirname, '..', 'cloud', 'sandbox', 'browser', 'main', 'project-browser', 'cloud-host.mts');
const text = (r: CallToolResult | null | undefined): string => (r?.content ?? []).map((c) => (c.type === 'text' ? c.text : '')).join('\n');

describe.skipIf(!chrome)('a project\'s browser carried to the cloud, on a real Chrome for Testing', { timeout: 120_000 }, () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'cloud-browser-'));
  const sock = path.join(root, 'host.sock');
  const given = path.join(root, 'given.json');
  const out = path.join(root, 'renewed.json');
  let site: http.Server;
  let on: string;
  let off: string;
  let host: ReturnType<typeof serveBrowser>;
  let copy: ChildProcessWithoutNullStreams;
  let nextId = 1;
  const waiting = new Map<number, (result: unknown) => void>();

  const ask = <T,>(method: Parameters<typeof askHost>[1]['method'], more: Record<string, unknown> = {}) => askHost<T>(sock, { method, ...more }, { ms: 60_000 });
  const rpc = <T,>(method: string, params: Record<string, unknown> = {}): Promise<T> => new Promise((resolve) => {
    const id = nextId++;
    waiting.set(id, resolve as (r: unknown) => void);
    copy.stdin.write(`${JSON.stringify({ jsonrpc: '2.0', id, method, params })}\n`);
  });
  const callCopy = (name: string, args: Record<string, unknown> = {}) => rpc<CallToolResult>('tools/call', { name, arguments: args });
  const sessionHere = async () => ((await ask<{ state: StorageState }>('state'))?.state.cookies ?? []).find((c: Cookie) => c.name === 'session')?.value;

  beforeAll(async () => {
    site = http.createServer((req, res) => {
      const cookie = req.headers.cookie ?? '';
      const send = (body: string, headers: Record<string, string | string[]> = {}) => { res.writeHead(200, { 'content-type': 'text/html', ...headers }); res.end(body); };
      if (req.url === '/sign-in') return send('<h1>Signed in</h1>', { 'set-cookie': 'session=owner; Max-Age=86400; Path=/' });
      if (req.url === '/renew') return send('<h1>Renewed</h1>', { 'set-cookie': ['session=renewed; Max-Age=86400; Path=/', 'fresh=made-there; Max-Age=86400; Path=/'] });
      return send(`<h1>Page ${req.url}</h1><p>${/session=(owner|renewed)/.test(cookie) ? 'Signed in as the owner' : 'Not signed in'}</p>`);
    });
    await new Promise<void>((resolve) => site.listen(0, '127.0.0.1', resolve));
    const { port } = site.address() as AddressInfo;
    on = `http://localhost:${port}`;
    off = `http://127.0.0.1:${port}`;

    const dir = path.join(root, 'browser');
    writeProject(dir, { slug: 'oms', sites: ['localhost'], pages: path.join(root, 'pages') });
    host = serveBrowser({ dir, sock, appPid: process.pid });
    await host.started;
    // The owner's sign-in, made in the Mac's copy of the project's browser.
    await askHost(sock, { method: 'tools/call', thread: 'here', params: { name: 'browser_navigate', arguments: { url: `${on}/sign-in` } } }, { ms: 60_000 });
  });

  afterAll(async () => {
    copy?.kill();
    await host?.close();
    await new Promise((resolve) => site.close(resolve));
    fs.rmSync(root, { recursive: true, force: true });
  });

  it('hands over the sign-ins its profile holds', async () => {
    const got = await ask<{ ok: boolean; sites: string[]; state: StorageState }>('state');
    expect(got?.ok).toBe(true);
    expect(got?.sites).toEqual(['localhost']);
    expect(got?.state.cookies.map((c) => `${c.name}=${c.value}`)).toContain('session=owner');
    fs.writeFileSync(given, JSON.stringify(got!.state));
  });

  it('is served by the copy the cloud image carries, signed in as the owner', async () => {
    copy = spawn(process.execPath, [COPY], {
      env: { ...process.env, PROJECT_BROWSER_SITES: 'localhost', PROJECT_BROWSER_STATE: given, PROJECT_BROWSER_OUT: out, PROJECT_BROWSER_CHROME: chrome!, PROJECT_BROWSER_THREAD: 'w-cloud' },
      stdio: ['pipe', 'pipe', 'pipe'],
    });
    readline.createInterface({ input: copy.stdout }).on('line', (line) => {
      const msg = JSON.parse(line);
      waiting.get(msg.id)?.(msg.result ?? msg.error);
      waiting.delete(msg.id);
    });
    await rpc('initialize', { protocolVersion: '2024-11-05' });
    const { tools } = await rpc<{ tools: Tool[] }>('tools/list');
    expect(tools.map((t) => t.name)).toContain('browser_navigate');
    expect(tools.map((t) => t.name)).not.toContain('browser_evaluate');
    await callCopy('browser_navigate', { url: `${on}/board` });
    expect(text(await callCopy('browser_snapshot'))).toContain('Signed in as the owner');
  });

  it('opens nothing off the project\'s sites', async () => {
    const refused = await callCopy('browser_navigate', { url: `${off}/elsewhere` });
    expect(refused.isError).toBe(true);
    expect(text(refused)).toMatch(/not one of this project's sites/);
  });

  it('writes the sign-in the site renewed, and not a cookie it only made there', async () => {
    await callCopy('browser_navigate', { url: `${on}/renew` });
    const renewed = JSON.parse(fs.readFileSync(out, 'utf8')).cookies as Cookie[];
    expect(renewed.map((c) => `${c.name}=${c.value}`)).toEqual(['session=renewed']);
  });

  it('is taken back here, with nothing the profile did not hold', async () => {
    expect(await sessionHere()).toBe('owner');
    const renewed = JSON.parse(fs.readFileSync(out, 'utf8')).cookies as Cookie[];
    const forged = { ...renewed[0], name: 'forged', value: 'x' };
    const got = await ask<{ ok: boolean; taken: number }>('take-cookies', { cookies: [...renewed, forged] });
    expect(got).toEqual({ ok: true, taken: 1 });
    expect(await sessionHere()).toBe('renewed');
    const names = ((await ask<{ state: StorageState }>('state'))?.state.cookies ?? []).map((c) => c.name);
    expect(names).not.toContain('forged');
  });
});
