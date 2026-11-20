// 2026-11-20, the first real Antigravity thread in the cloud: its first turn
// worked and its work came home, but the reply failed with "session/resume:
// Failed to read session". Measured: inside the cloud the conversation is a
// plain SQLite database (page size 32768, WAL), and the copy that came home
// had every byte that is not UTF-8 turned into EF BF BD. The sandbox takes the
// sign-in's secrets out of what comes home, reading each file as text; the
// Antigravity sign-in's "secrets" were every long string in its files, its
// project id among them, and that id is in every conversation. Three changes:
// only token values are secret, a file that is not text is masked byte for
// byte, and the conversation a reply resumes goes up with the run's start.
import { describe, it, expect, afterAll, vi } from 'vitest';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { EventEmitter } from 'node:events';
import { redactFolder, agyLoginSecrets, REDACTED } from '../cloud/sandbox/server.mjs';
import { agyConversationFor } from '../main/cloud/run.mts';
import { Supervisor } from '../main/supervisor.mts';
import { createCodexApprovals } from '../main/codex-approvals.mts';

const dirs: string[] = [];
const tmp = () => { const d = fs.mkdtempSync(path.join(os.tmpdir(), 'agy-reply-')); dirs.push(d); return d; };
afterAll(() => { for (const d of dirs) fs.rmSync(d, { recursive: true, force: true }); });

const TOKEN = 'ya29.fake-access-token-' + 'A'.repeat(30); // public-check: allow (made up)

describe('what comes home from the cloud', () => {
  it('keeps a database whole, masking a secret in it byte for byte', () => {
    const dir = tmp();
    const db = Buffer.concat([Buffer.from('SQLite format 3\0'), Buffer.from([0x80, 0x00, 0x02, 0x02, 0xff, 0xfe]), Buffer.from(`..${TOKEN}..`), Buffer.from([0x81, 0x00])]);
    fs.writeFileSync(path.join(dir, 'c.db'), db);
    redactFolder(dir, [TOKEN]);
    const out = fs.readFileSync(path.join(dir, 'c.db'));
    expect(out.length).toBe(db.length);
    expect(out.subarray(0, 22)).toEqual(db.subarray(0, 22));
    expect(out.includes(Buffer.from(TOKEN))).toBe(false);
    expect(out.includes(Buffer.from('*'.repeat(TOKEN.length)))).toBe(true);
  });

  it('still names the secret it took out of a transcript, and leaves a file without one alone', () => {
    const dir = tmp();
    fs.writeFileSync(path.join(dir, 't.jsonl'), `{"said":"${TOKEN}"}\n`);
    const plain = Buffer.from([0x80, 0x81, 0x82]);
    fs.writeFileSync(path.join(dir, 'other.db'), plain);
    redactFolder(dir, [TOKEN]);
    expect(fs.readFileSync(path.join(dir, 't.jsonl'), 'utf8')).toBe(`{"said":"${REDACTED}"}\n`);
    expect(fs.readFileSync(path.join(dir, 'other.db'))).toEqual(plain);
  });
});

describe('what in an Antigravity sign-in is secret', () => {
  it('is its tokens, not every long string in it', () => {
    const json = (v: unknown) => Buffer.from(JSON.stringify(v));
    expect(agyLoginSecrets('.gemini/config/projects/default-cli-project.json', json({ projectId: 'gen-lang-client-0123456789abcdef' }))).toEqual([]);
    expect(agyLoginSecrets('.gemini/antigravity-cli/cache/auth.json', json({ access_token: TOKEN, expiry: '2026-11-20T15:29:06.123456Z', email: 'someone@example.test' }))).toEqual([TOKEN]);
    expect(agyLoginSecrets('.gemini/a.json', json({ token: { value: TOKEN } }))).toEqual([TOKEN]);
  });

  it('is the whole file when the file is a token', () => {
    expect(agyLoginSecrets('.gemini/antigravity-cli/antigravity-oauth-token', Buffer.from(`${TOKEN}\n`))).toEqual([TOKEN]);
    expect(agyLoginSecrets('.gemini/antigravity-cli/installation_id', Buffer.from('3b5c6f0e-1111-2222-3333-444455556666'))).toEqual([]);
  });
});

describe('a reply’s conversation', () => {
  it('is found by its id under the agy home, and not for an id that is a path', () => {
    expect(agyConversationFor('/h/agy', 'b3e72db1-f9ae-40a6-a27f')).toBe('/h/agy/antigravity-cli/conversations/b3e72db1-f9ae-40a6-a27f.db');
    expect(agyConversationFor('/h/agy', '../escape')).toBeNull();
    expect(agyConversationFor(null, 'abc')).toBeNull();
  });

  it('is named to the relay by the reply that resumes it, and not by a first turn', async () => {
    const started = async (resumeId: string | null) => {
      const relay = new EventEmitter() as EventEmitter & { stdout: EventEmitter; stderr: EventEmitter; stdin: { write(t: string, cb?: () => void): boolean; end(): void }; kill(): void; pid: number };
      relay.stdout = new EventEmitter(); relay.stderr = new EventEmitter(); relay.pid = 1; relay.kill = () => {};
      relay.stdin = { write: (_t, cb) => { cb?.(); return true; }, end: () => {} };
      let env: NodeJS.ProcessEnv = {};
      const sup = Object.assign(Object.create(Supervisor.prototype), {
        appDir: '/app', userDir: '/u', config: { storeRoot: '/s', accountId: 'acct', antigravityBin: null },
        storeMcpCommand: () => null, _cloudDocsEnv: () => ({}),
        _codexApprovals: () => createCodexApprovals({ storeRoot: tmp() }),
        _spawnCloudRelay: (_a: string[], o: { env: NodeJS.ProcessEnv }) => { env = o.env; return relay; },
      });
      const worker = sup._spawnAntigravityWorker({ args: [], prompt: 'Go on', resumeId }, { cwd: '/w', item: { id: 'w-1', product: 'shop' }, env: {}, cloud: true });
      await vi.waitFor(() => expect(env.ZERO_ITEM).toBe('w-1'));
      worker.kill();
      return env.AGENTBOX_AGY_RESUME ?? null;
    };
    expect(await started('b3e72db1-f9ae')).toBe('b3e72db1-f9ae');
    expect(await started(null)).toBeNull();
  });
});
