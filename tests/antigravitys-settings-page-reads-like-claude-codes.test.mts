// 2026-11-20, the owner on the Antigravity page: "não está igual ao do claude
// code" (one card saying "Google account, Signed in") and "não mostra um menu
// de cloud". Its page is now Claude Code's: the account by its email with In
// use and a tag per place it works, Add to cloud where it is not there yet,
// Cloud once it is, and Permissions in agy's own modes, here and in the cloud.
// Whose account it is comes from agy's log: no agy command says, and
// `agy --log-file <f> models` wrote "authenticated successfully as <email>"
// (agy 1.5.13).
import { describe, it, expect, vi } from 'vitest';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { EventEmitter } from 'node:events';
import { antigravityTiles } from '../renderer/src/components/Settings.tsx';
import type { CloudStatus } from '../renderer/src/cloud-menu.ts';
import { accountInAgyLog, readAntigravityModels } from '../main/antigravity-models.mts';
import { antigravityState, setWorkspaceSetting } from '../main/settings.mts';
import { cloudAntigravityMode, CLOUD_SAME } from '../main/cloud-permission.mts';
import { ANTIGRAVITY_MODES, ANTIGRAVITY_MODE_ORDER } from '../shared/antigravity-modes.mts';
import { Supervisor } from '../main/supervisor.mts';
import { createCodexApprovals } from '../main/codex-approvals.mts';
import { recordCloudLogin } from '../main/cloud/logins.mts';

const tmp = () => fs.mkdtempSync(path.join(os.tmpdir(), 'agy-page-'));
const month = { runs: 0, agentHours: 0, estimatedCost: 0, costPerAgentHour: null };
const cloudWith = (accounts: unknown[] = []) => ({ setUp: true, mode: 'full', running: 0, month, accounts, signIns: [] }) as unknown as CloudStatus;
const login = (key: string, name: string | null, state = 'ready') => ({ engine: 'antigravity', key, name, ready: state === 'ready', state });

describe('the Antigravity account’s tile', () => {
  const acts = () => ({ onSignIn: vi.fn(), onCloudSignIn: vi.fn() });

  it('is named by its email and in use, and offers the cloud where it is not there yet', () => {
    const a = acts();
    const [tile, ...rest] = antigravityTiles({ signedIn: true, email: 'me@example.test' }, cloudWith(), a);
    expect(rest).toEqual([]);
    expect(tile).toMatchObject({ name: 'me@example.test', inUse: true, places: ['here'], about: [], meters: false });
    expect(tile.cloudAct?.word).toBe('Add to cloud');
    tile.cloudAct!.onUse();
    expect(a.onCloudSignIn).toHaveBeenCalledWith(null, 'me@example.test');
  });

  it('says both places once the cloud has the same account, and offers nothing more', () => {
    const [tile] = antigravityTiles({ signedIn: true, email: 'Me@Example.test' }, cloudWith([login('g1', 'me@example.test')]), acts());
    expect(tile).toMatchObject({ places: ['here', 'cloud'], cloudAct: null });
  });

  it('offers to sign in to the cloud again when that sign-in stopped working', () => {
    const a = acts();
    const [tile] = antigravityTiles({ signedIn: true, email: 'me@example.test' }, cloudWith([login('g1', 'me@example.test', 'broken')]), a);
    expect(tile.places).toEqual(['here']);
    expect(tile.cloudAct?.word).toBe('Sign in to cloud again');
    tile.cloudAct!.onUse();
    expect(a.onCloudSignIn).toHaveBeenCalledWith('g1', 'me@example.test');
  });

  it('takes the first cloud sign-in as its own while whose it is is unknown, and draws another account apart', () => {
    const tiles = antigravityTiles({ signedIn: true, email: null }, cloudWith([login('g1', 'me@example.test'), login('g2', 'work@example.test')]), acts());
    expect(tiles.map((t) => [t.name, t.places])).toEqual([['me@example.test', ['here', 'cloud']], ['work@example.test', ['cloud']]]);
  });

  it('has no place tags without a cloud, and a Sign in button when signed out', () => {
    const a = acts();
    expect(antigravityTiles({ signedIn: true, email: 'me@example.test' }, null, a)[0]).toMatchObject({ places: [], cloudAct: null });
    const [out] = antigravityTiles({ signedIn: false }, cloudWith(), a);
    expect(out).toMatchObject({ name: 'Google account', about: ['Not signed in'], inUse: false, places: [], cloudAct: null });
    out.use!.onUse();
    expect(a.onSignIn).toHaveBeenCalled();
  });
});

describe('whose Antigravity sign-in it is', () => {
  it('is read from the line agy logs when it signs in', () => {
    expect(accountInAgyLog('I1120 01:01:09.391675 40 server_oauth.go:226] OAuth: authenticated successfully as me@example.test\n')).toBe('me@example.test');
    expect(accountInAgyLog('nothing here\n')).toBeNull();
  });

  it('comes from the log file the sign-in check gives agy, which is then deleted', async () => {
    const dir = tmp();
    const logFile = path.join(dir, 'agy.log');
    let asked: string[] = [];
    const spawn = (_file: string, args: string[]) => {
      asked = args;
      const child = new EventEmitter() as EventEmitter & { stdout: EventEmitter; kill(): void };
      child.stdout = new EventEmitter();
      child.kill = () => {};
      setImmediate(() => {
        fs.writeFileSync(args[1], 'OAuth: authenticated successfully as me@example.test\n');
        child.stdout.emit('data', Buffer.from('gemini-4-neon\tGemini 4 Neon\n'));
        child.emit('close', 0);
      });
      return child as never;
    };
    const answer = await readAntigravityModels('/u/agy', { spawn, logFile });
    expect(asked).toEqual(['--log-file', logFile, 'models']);
    expect(answer).toEqual({ signedIn: true, email: 'me@example.test' });
    expect(fs.existsSync(logFile)).toBe(false);
    fs.rmSync(dir, { recursive: true, force: true });
  });

  it('reaches the page only while it is signed in', () => {
    expect(antigravityState({ antigravityBin: '/u/agy', antigravitySignedIn: true, antigravityEmail: 'me@example.test' }, { search: false }).email).toBe('me@example.test');
    expect(antigravityState({ antigravityBin: '/u/agy', antigravitySignedIn: false, antigravityEmail: 'me@example.test' }, { search: false }).email).toBeNull();
  });

  it('picks its cloud sign-in by that email, and none when the cloud has only another account', () => {
    const home = tmp();
    recordCloudLogin('antigravity', 'acct-g1', { id: 'id-g1', email: 'work@example.test' }, { home });
    recordCloudLogin('antigravity', 'acct-g2', { id: 'id-g2', email: 'me@example.test' }, { home });
    const sup = (email: string | null) => Object.assign(new Supervisor({ home, storeRoot: home, maxConcurrentSessions: 1, authProfiles: ['default'], antigravityEmail: email }, { listItems: () => [], listProducts: () => [] }, '/nonexistent-app'), {
      _profilesFor: () => ['default'],
    });
    expect(sup('ME@example.test')._cloudAccountOfThisMac('antigravity')).toBe('acct-g2');
    expect(sup('other@example.test')._cloudAccountOfThisMac('antigravity')).toBeNull();
    expect(sup(null)._cloudAccountOfThisMac('antigravity')).toBe('acct-g1');
    fs.rmSync(home, { recursive: true, force: true });
  });
});

describe('Antigravity’s permission modes', () => {
  it('are agy’s own four, Auto first and the default, and Full access last', () => {
    expect(ANTIGRAVITY_MODE_ORDER).toEqual(['auto', 'accept-edits', 'default', 'plan', 'full-access']);
    expect(ANTIGRAVITY_MODE_ORDER.map((m) => ANTIGRAVITY_MODES[m].label)).toEqual(['Auto', 'Accept edits', 'Default', 'Plan', 'Full access']);
    expect(antigravityState({ antigravityBin: '/u/agy' }, { search: false })).toMatchObject({ mode: 'auto', cloudMode: CLOUD_SAME });
  });

  it('in the cloud are this computer’s unless set, and a reply’s own mode wins', () => {
    expect(cloudAntigravityMode(null, null, 'accept-edits')).toBe('accept-edits');
    expect(cloudAntigravityMode(CLOUD_SAME, null, 'auto')).toBe('auto');
    expect(cloudAntigravityMode('plan', null, 'auto')).toBe('plan');
    expect(cloudAntigravityMode('plan', 'default', 'auto')).toBe('default');
  });

  it('are saved, and a word agy does not know is refused', () => {
    const dir = tmp();
    const w = { config: { appDir: dir } as Record<string, unknown>, supervisor: { onChange() {} }, store: {} };
    setWorkspaceSetting(w as never, { key: 'antigravityMode', value: 'accept-edits' });
    setWorkspaceSetting(w as never, { key: 'cloudAntigravityMode', value: CLOUD_SAME });
    expect(w.config).toMatchObject({ antigravityMode: 'accept-edits', cloudAntigravityMode: CLOUD_SAME });
    expect(() => setWorkspaceSetting(w as never, { key: 'antigravityMode', value: 'bypassPermissions' })).toThrow();
    expect(() => setWorkspaceSetting(w as never, { key: 'cloudAntigravityMode', value: 'acceptEdits' })).toThrow();
    fs.rmSync(dir, { recursive: true, force: true });
  });

  // The relay as the worker sees it, answering every request the worker makes.
  function fakeRelay() {
    const proc = new EventEmitter() as EventEmitter & { stdout: EventEmitter; stderr: EventEmitter; sent: { id?: number; method?: string; params?: { modeId?: string } }[]; pid: number; stdin: { write(t: string, cb?: () => void): boolean; end(): void }; kill(): void };
    proc.stdout = new EventEmitter();
    proc.stderr = new EventEmitter();
    proc.sent = [];
    proc.pid = 7;
    const say = (m: unknown) => proc.stdout.emit('data', Buffer.from(`${JSON.stringify(m)}\n`));
    const modes = { currentModeId: 'default', availableModes: ANTIGRAVITY_MODE_ORDER.map((id) => ({ id })) };
    proc.stdin = {
      write: (t, cb) => {
        const m = JSON.parse(t);
        proc.sent.push(m);
        cb?.();
        const result = m.method === 'initialize' ? { protocolVersion: 1, agentCapabilities: { _meta: { antigravity: { systemPrompt: { append: true } } } } }
          : m.method === 'session/new' ? { sessionId: 's', modes }
          : m.method === 'session/prompt' ? { stopReason: 'end_turn' } : {};
        if (m.id != null) queueMicrotask(() => say({ jsonrpc: '2.0', id: m.id, result }));
        return true;
      },
      end: () => {},
    };
    proc.kill = () => {};
    return proc;
  }
  const runStarted = async (config: Record<string, unknown>, { cloud = true, answerMode = null }: { cloud?: boolean; answerMode?: string | null } = {}) => {
    const relay = fakeRelay();
    let args: string[] = [];
    const sup = Object.assign(Object.create(Supervisor.prototype), {
      appDir: '/app', userDir: '/u', config: { storeRoot: '/s', accountId: 'acct', antigravityBin: null, ...config },
      storeMcpCommand: () => null, _cloudDocsEnv: () => ({}),
      _codexApprovals: () => createCodexApprovals({ storeRoot: tmp() }),
      _spawnCloudRelay: (a: string[]) => { args = a; return relay; },
    });
    sup._spawnAntigravityWorker({ args: [], prompt: 'Fix it', answerMode }, { cwd: '/w', item: { id: 'w-1', product: 'shop' }, env: {}, cloud });
    await vi.waitFor(() => expect(relay.sent.some((m) => m.method === 'session/prompt')).toBe(true));
    return { mode: relay.sent.find((m) => m.method === 'session/set_mode')?.params?.modeId ?? null, skips: args.includes('--dangerously-skip-permissions') };
  };

  it('start a run in the mode set here, or the cloud’s own, or the reply’s', async () => {
    expect(await runStarted({})).toEqual({ mode: 'auto', skips: false });
    expect(await runStarted({ antigravityMode: 'accept-edits' })).toEqual({ mode: 'accept-edits', skips: false });
    expect(await runStarted({ antigravityMode: 'accept-edits', cloudAntigravityMode: 'plan' })).toEqual({ mode: 'plan', skips: false });
    expect(await runStarted({ antigravityMode: 'accept-edits', cloudAntigravityMode: 'plan' }, { answerMode: 'default' })).toEqual({ mode: null, skips: false });
  });

  // 2026-11-20: "o agy não tem um modo de full access igual o codex e claude?"
  it('run Full access as Auto on a server that asks nothing, here, in the cloud and from a reply', async () => {
    expect(await runStarted({ antigravityMode: 'full-access' })).toEqual({ mode: 'auto', skips: true });
    expect(await runStarted({ antigravityMode: 'auto', cloudAntigravityMode: 'full-access' })).toEqual({ mode: 'auto', skips: true });
    expect(await runStarted({ antigravityMode: 'full-access', cloudAntigravityMode: 'plan' })).toEqual({ mode: 'plan', skips: false });
    expect(await runStarted({}, { answerMode: 'bypassPermissions' })).toEqual({ mode: 'auto', skips: true });
    expect(await runStarted({ antigravityMode: 'full-access' }, { answerMode: 'plan' })).toEqual({ mode: 'plan', skips: false });
  });

  it('save Full access like any other', () => {
    const dir = tmp();
    const w = { config: { appDir: dir } as Record<string, unknown>, supervisor: { onChange() {} }, store: {} };
    setWorkspaceSetting(w as never, { key: 'antigravityMode', value: 'full-access' });
    setWorkspaceSetting(w as never, { key: 'cloudAntigravityMode', value: 'full-access' });
    expect(w.config).toMatchObject({ antigravityMode: 'full-access', cloudAntigravityMode: 'full-access' });
    expect(antigravityState(w.config as never, { search: false })).toMatchObject({ mode: 'full-access', cloudMode: 'full-access' });
    fs.rmSync(dir, { recursive: true, force: true });
  });
});
