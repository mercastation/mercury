// A SELF-HOSTED CLOUD CARRIES A PHONE REPLY WITH THE COMPUTER SHUT.
//
// The managed cloud answers a phone's reply when the computer is asleep
// (cloud/gateway/phone.mjs). The self-hosted server never did: it had no
// database to read the replies from, so on a server at home the phone said
// "waiting for your Mac" until the lid opened, with the kit the Mac had left
// sitting in the server's own bucket. Now a `phone` in its config names the
// database, read as the server's own role (cloud/selfhost/phone-role.sql)
// through the publishable key and in the schema the tables live in. Measured
// with the real server: its bucket on disk, its clock, a done.json put through
// the bucket's signed link over HTTP, and a stand-in database.
import { describe, it, expect, afterEach, vi } from 'vitest';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import http from 'node:http';
import crypto from 'node:crypto';
import { selfHostedCloud, phoneDatabase } from '../cloud/selfhost/server.mjs';
import { supabaseRest, ANSWERED_OUTCOME } from '../cloud/gateway/phone.mjs';
import { TERMS_VERSION } from '../cloud/gateway/terms.mjs';
import { KIT_VERSION, kitKey, runOfClaim } from '../shared/cloud-kit.mts';
import { threadSnapshot } from '../shared/phone-snapshot.mts';

const A = '33333333-3333-4333-8333-333333333333';
const DEVICE = 'a1b2c3d4e5f6';
const MIN = 60_000;
const REPLY = 'yes, ship it';
const ISSUER = 'https://db.example.test/auth/v1';
const TOKEN = 'ExponentPushToken[abcdefghij0123456789AB]';

const keys = crypto.generateKeyPairSync('ec', { namedCurve: 'P-256' });
const jwks = { keys: [{ ...keys.publicKey.export({ format: 'jwk' }), kid: 'k1', alg: 'ES256' }] };
const b64 = (o: unknown) => Buffer.from(JSON.stringify(o)).toString('base64url');
function sessionOf(sub: string) {
  const head = `${b64({ alg: 'ES256', kid: 'k1', typ: 'JWT' })}.${b64({ sub, role: 'authenticated', iss: ISSUER, exp: Math.floor(Date.now() / 1000) + 3600 })}`;
  return `${head}.${crypto.sign('sha256', Buffer.from(head), { key: keys.privateKey, dsaEncoding: 'ieee-p1363' }).toString('base64url')}`;
}

type Command = Record<string, unknown> & { id: string; created_at: string; claimed_at: string | null; claimed_by: string | null; taken_at: string | null; outcome: string | null };
function fakeDb() {
  const commands: Command[] = [];
  const threads = new Map<string, { snapshot: unknown }>();
  const listed: number[] = [];
  const at = (owner: string, device: string, project: string, item: string) => `${owner}|${device}|${project}|${item}`;
  return {
    commands, threads, listed,
    async waitingReplies({ after, before, limit }: { after: number; before: number; limit: number }) {
      listed.push(1);
      return commands.filter((c) => !c.taken_at && !c.claimed_at && Date.parse(c.created_at) > after && Date.parse(c.created_at) < before).slice(0, limit).map((c) => ({ ...c }));
    },
    async claim(id: string, by: string, when: string) {
      const c = commands.find((x) => x.id === id && !x.claimed_at && !x.taken_at);
      if (!c) return false;
      Object.assign(c, { claimed_at: when, claimed_by: by });
      return true;
    },
    async giveBack(id: string, by: string) {
      const c = commands.find((x) => x.id === id && x.claimed_by === by && !x.taken_at);
      if (c) Object.assign(c, { claimed_at: null, claimed_by: null });
    },
    async taken(id: string, outcome: string, when: string) {
      const c = commands.find((x) => x.id === id && !x.taken_at);
      if (c) Object.assign(c, { taken_at: when, outcome });
    },
    async thread(o: string, d: string, p: string, i: string) { return threads.get(at(o, d, p, i)) ?? null; },
    async writeThread(o: string, d: string, p: string, i: string, snapshot: unknown) { threads.set(at(o, d, p, i), { snapshot }); },
    async pushDevices(owner: string) { return owner === A ? [{ token: TOKEN }] : []; },
    async dropPushDevice() {},
  };
}

let tmp: string; let server: http.Server | null = null;
afterEach(async () => {
  vi.useRealTimers();
  if (server) await new Promise((r) => server!.close(r));
  server = null;
  if (tmp) fs.rmSync(tmp, { recursive: true, force: true });
});

async function setup({ phone = true } = {}) {
  let clock = Date.UTC(2026, 10, 15, 12, 0, 0);
  tmp = fs.mkdtempSync(path.join(os.tmpdir(), 'selfhost-phone-'));
  const db = fakeDb();
  const sent: { sessionId: string; payload: Record<string, unknown> }[] = [];
  const pushed: unknown[][] = [];
  let cloud: Awaited<ReturnType<typeof selfHostedCloud>> | null = null;
  server = http.createServer((req, res) => cloud!.handler(req, res));
  await new Promise<void>((r) => server!.listen(0, '127.0.0.1', r));
  const base = `http://127.0.0.1:${(server.address() as { port: number }).port}`;
  cloud = await selfHostedCloud({
    dataDir: tmp, secret: 'only-the-server-knows', publicUrl: base, jwks, people: [A], issuer: ISSUER, agents: 2,
    sessions: {
      async invoke(sessionId: string, payload: Record<string, unknown>) { sent.push({ sessionId, payload }); return { ok: true }; },
      async stop() {},
      async sweep() {},
    },
    phone: phone ? { db, sendPush: async (messages: unknown[]) => { pushed.push(messages); return messages.map(() => ({ status: 'ok' })); } } : null,
    now: () => clock,
  });
  await cloud.gw.handle({ method: 'POST', path: '/v1/devices', headers: { authorization: `Bearer ${sessionOf(A)}` }, body: JSON.stringify({ acceptedTerms: TERMS_VERSION }) });
  const kit = {
    v: KIT_VERSION, engine: 'claude', madeAt: clock - MIN, device: DEVICE, project: 'alpha', item: 'w-1',
    cwd: '/home/me/dev/alpha/.claude/worktrees/w-1', branch: 'agentbox/w-1', startSha: 'abc123',
    argv: ['claude', '--resume', 's-1'], env: {}, bundleKey: `runs/kits/${DEVICE}/alpha/w-1/in.bundle`, files: [],
    login: { engine: 'claude', id: 'acct-1', key: 'logins/claude/acct-1.json' }, bringDirs: [{ name: 'projects.tgz', dir: '/home/me/.claude/projects/-x' }],
    frame: { before: 'What they said:\n\n', after: '' }, note: '# In the cloud',
  };
  // Where the Mac leaves it: through the bucket, under the person's own runs.
  await cloud.bucket.objects.put(`runs/u-${A}/kits/${kitKey(DEVICE, 'alpha', 'w-1').slice('runs/kits/'.length)}`, JSON.stringify(kit));
  db.threads.set(`${A}|${DEVICE}|alpha|w-1`, { snapshot: threadSnapshot({ id: 'w-1', title: 'Ship it', status: 'blocked', result: 'Ready to ship?', updatedAt: clock - 3600_000 }, []) });
  db.commands.push({ id: 'c1', owner: A, device: DEVICE, project: 'alpha', item: 'w-1', kind: 'reply', payload: { text: REPLY }, created_at: new Date(clock - 2 * MIN).toISOString(), claimed_at: null, claimed_by: null, taken_at: null, outcome: null });
  // A run's files go into the bucket as a sandbox puts them: through a signed link, over HTTP.
  const putThrough = async (key: string, body: string) => {
    const res = await fetch(await cloud!.bucket.links.put(key, 300), { method: 'PUT', body });
    expect(res.ok).toBe(true);
  };
  return { cloud, db, sent, pushed, putThrough, tick: (ms: number) => { clock += ms; } };
}

describe('a self-hosted cloud with the phone in its config', () => {
  it('starts a reply the computer left from the kit in its own bucket, on its own clock', async () => {
    vi.useFakeTimers({ toFake: ['setInterval', 'clearInterval'] });
    const t = await setup();
    t.cloud.startTimers();
    await vi.advanceTimersByTimeAsync(MIN);
    t.cloud.stopTimers();
    const starts = t.sent.filter((s) => s.payload.action === 'start');
    expect(starts).toHaveLength(1);
    expect(starts[0].payload).toMatchObject({ cwd: '/home/me/dev/alpha/.claude/worktrees/w-1', argv: ['claude', '--resume', 's-1'] });
    expect(runOfClaim(t.db.commands[0].claimed_by)).toBe(starts[0].payload.runId);
  });

  it('puts the agent\'s answer on the phone once the run\'s done.json lands through its door, and sends one alert', async () => {
    const t = await setup();
    await t.cloud.gw.phoneSweep();
    const runId = runOfClaim(t.db.commands[0].claimed_by);
    t.tick(10 * MIN);
    const line = `${JSON.stringify({ type: 'result', result: 'Shipped it.', is_error: false })}\n`;
    await t.putThrough(`runs/u-${A}/${runId}/events.jsonl`, JSON.stringify({ seq: 1, kind: 'stdout', data: Buffer.from(line).toString('base64') }));
    await t.putThrough(`runs/u-${A}/${runId}/done.json`, '{}');
    await vi.waitFor(() => expect(t.db.commands[0].outcome).toBe(ANSWERED_OUTCOME));
    const snap = t.db.threads.get(`${A}|${DEVICE}|alpha|w-1`)!.snapshot as { item: Record<string, unknown> };
    expect(snap.item).toMatchObject({ answer: REPLY, result: 'Shipped it.' });
    expect(t.pushed).toHaveLength(1);
    expect(t.pushed[0][0]).toMatchObject({ to: TOKEN, title: 'Ship it' });
  });
});

describe('a self-hosted cloud without it', () => {
  it('leaves the reply for the computer and never asks a database', async () => {
    vi.useFakeTimers({ toFake: ['setInterval', 'clearInterval'] });
    const t = await setup({ phone: false });
    t.cloud.startTimers();
    await vi.advanceTimersByTimeAsync(5 * MIN);
    t.cloud.stopTimers();
    expect(await t.cloud.gw.phoneSweep()).toMatchObject({ ok: false });
    expect(t.db.listed).toEqual([]);
    expect(t.db.commands[0].claimed_at).toBeNull();
  });
});

describe('the database the config names', () => {
  type Asked = { url: string; headers: Record<string, string> };
  const recorder = () => {
    const asked: Asked[] = [];
    const fetchImpl = async (url: string, init: { headers: Record<string, string> }) => {
      asked.push({ url, headers: init.headers });
      return { ok: true, status: 200, text: async () => '[]', json: async () => ({ signedURL: '/object/sign/phone-files/x?token=t' }) };
    };
    return { asked, fetchImpl };
  };

  it('is read as the server\'s role, through the publishable key, in the schema it names', async () => {
    const { asked, fetchImpl } = recorder();
    const db = phoneDatabase({ url: 'https://api.example.test/', anonKey: 'sb_publishable_x', keyFile: '/etc/cloud/phone-key', schema: 'mercury' },
      { readFile: (f: string) => (f === '/etc/cloud/phone-key' ? 'role-token\n' : ''), fetchImpl });
    await db.waitingReplies({ after: 0, before: 1, limit: 5 });
    await db.claim('c1', 'cloud:r-1', new Date(0).toISOString());
    await db.signPhoneFile(A, DEVICE, `${'a'.repeat(64)}.png`, 60);
    expect(asked.map((a) => a.url.split('?')[0])).toEqual([
      'https://api.example.test/rest/v1/phone_commands',
      'https://api.example.test/rest/v1/phone_commands',
      `https://api.example.test/storage/v1/object/sign/phone-files/${A}/${DEVICE}/${'a'.repeat(64)}.png`,
    ]);
    for (const a of asked) expect(a.headers).toMatchObject({ apikey: 'sb_publishable_x', authorization: 'Bearer role-token' });
    expect(asked[0].headers['accept-profile']).toBe('mercury');
    expect(asked[1].headers['content-profile']).toBe('mercury');
  });

  it('is refused without its address, its key or the file with the role\'s token', () => {
    const readFile = () => 'role-token';
    expect(() => phoneDatabase({ anonKey: 'k', keyFile: '/f' }, { readFile })).toThrow(/url, anonKey and keyFile/);
    expect(() => phoneDatabase({ url: 'https://x', keyFile: '/f' }, { readFile })).toThrow(/url, anonKey and keyFile/);
    expect(() => phoneDatabase({ url: 'https://x', anonKey: 'k' }, { readFile })).toThrow(/url, anonKey and keyFile/);
    expect(() => phoneDatabase({ url: 'https://x', anonKey: 'k', keyFile: '/f' }, { readFile: () => '\n' })).toThrow(/empty/);
  });

  it('is asked as before by the managed cloud: the service key twice and no schema', async () => {
    const { asked, fetchImpl } = recorder();
    await supabaseRest({ url: 'https://abc.supabase.co', serviceKey: 'svc', fetchImpl }).waitingReplies({ after: 0, before: 1, limit: 5 });
    expect(asked[0].headers).toMatchObject({ apikey: 'svc', authorization: 'Bearer svc' });
    expect(asked[0].headers).not.toHaveProperty('accept-profile');
  });
});
