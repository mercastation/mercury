// 2026-11-20: an Antigravity thread in the cloud is the same ACP worker as on
// the Mac, its `agy acp` started in the cloud machine through the relay. Its
// tool servers are named inside `session/new` and `session/resume`, so on the
// way up each becomes the sandbox's stub and the real one runs on the Mac, as
// Codex's do. The cloud note joins its appended instructions. A conversation
// lives in one file: measured on agy 1.5.13, a fresh agy home given only
// `antigravity-cli/conversations/<id>.db` resumed it (it remembered the word),
// so that file goes up for a resume and that folder comes home.
import { describe, it, expect, afterAll, vi } from 'vitest';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { EventEmitter } from 'node:events';
import { acpLineForTheCloud, bringDirsFor, cloudEngineOf, LOGIN_TROUBLE } from '../main/cloud/run.mts';
import { Supervisor } from '../main/supervisor.mts';
import { createCodexApprovals } from '../main/codex-approvals.mts';

const dirs: string[] = [];
afterAll(() => { for (const d of dirs) fs.rmSync(d, { recursive: true, force: true }); });

const store = { name: 'mercury', command: '/app/mcp/mercury-mcp.sh', args: [], env: [{ name: 'ZERO_ITEM', value: 'w-1' }, { name: 'ZERO_PRODUCT', value: 'shop' }] };
const line = (msg: unknown) => `${JSON.stringify(msg)}\n`;

describe('an Antigravity line on its way to the cloud', () => {
  it('swaps each tool server for the stub, keeps the real one here, and adds the cloud note to its instructions', () => {
    const keep: Record<string, unknown> = {};
    const out = acpLineForTheCloud(line({ jsonrpc: '2.0', id: 2, method: 'session/new', params: { cwd: '/w', mcpServers: [store], _meta: { antigravity: { systemPrompt: { append: 'The rules' } } } } }), { agyHome: '/h/agy', keep, note: 'You are in the cloud.' });
    const sent = JSON.parse(out.line);
    expect(out.line.endsWith('\n')).toBe(true);
    expect(sent.params.mcpServers).toEqual([{ name: 'mercury', command: '/usr/local/bin/node', args: ['/opt/agentbox/mcp-stub.mjs', 'mercury'], env: [{ name: 'AGENTBOX_BRIDGE_SOCK', value: '/tmp/agentbox-bridge.sock' }] }]);
    expect(keep.mercury).toEqual({ command: '/app/mcp/mercury-mcp.sh', args: [], env: { ZERO_ITEM: 'w-1', ZERO_PRODUCT: 'shop' } });
    expect(sent.params._meta.antigravity.systemPrompt.append).toBe('The rules\n\nYou are in the cloud.');
    expect(out.files).toEqual([]);
  });

  it('gives instructions to a session that had none, so it is told it is in the cloud', () => {
    const out = acpLineForTheCloud(line({ jsonrpc: '2.0', id: 2, method: 'session/new', params: { cwd: '/w', mcpServers: [] } }), { agyHome: '/h/agy', keep: {}, note: 'You are in the cloud.' });
    expect(JSON.parse(out.line).params._meta).toEqual({ antigravity: { systemPrompt: { append: 'You are in the cloud.' } } });
  });

  it('sends up the conversation a resume names, when this Mac has it', () => {
    const home = fs.mkdtempSync(path.join(os.tmpdir(), 'agy-cloud-'));
    dirs.push(home);
    const db = path.join(home, 'antigravity-cli', 'conversations', 'sess-1.db');
    fs.mkdirSync(path.dirname(db), { recursive: true });
    fs.writeFileSync(db, 'sqlite');
    const resume = (sessionId: string) => acpLineForTheCloud(line({ jsonrpc: '2.0', id: 2, method: 'session/resume', params: { sessionId, cwd: '/w', mcpServers: [store] } }), { agyHome: home, keep: {}, note: null });
    expect(resume('sess-1').files).toEqual([db]);
    expect(resume('sess-2').files).toEqual([]);
    expect(resume('../escape').files).toEqual([]);
  });

  it('passes every other line exactly as it came', () => {
    const prompt = line({ jsonrpc: '2.0', id: 3, method: 'session/prompt', params: { sessionId: 's', prompt: [{ type: 'text', text: 'hi' }] } });
    expect(acpLineForTheCloud(prompt, { agyHome: '/h', keep: {}, note: 'n' }).line).toBe(prompt);
    expect(acpLineForTheCloud('not json\n', { agyHome: '/h', keep: {}, note: 'n' }).line).toBe('not json\n');
  });
});

describe('an Antigravity worker in the cloud', () => {
  // The relay as the worker sees it: a process speaking ACP on its pipes.
  function fakeRelay() {
    const proc = new EventEmitter() as EventEmitter & { stdout: EventEmitter; stderr: EventEmitter; sent: { id?: number; method?: string }[]; ended: boolean; killed: boolean; pid: number; stdin: { write(t: string, cb?: () => void): boolean; end(): void }; kill(): void; reply(method: string, result: unknown): void };
    proc.stdout = new EventEmitter();
    proc.stderr = new EventEmitter();
    proc.sent = [];
    proc.ended = false;
    proc.killed = false;
    proc.pid = 77;
    const say = (m: unknown) => proc.stdout.emit('data', Buffer.from(`${JSON.stringify(m)}\n`));
    proc.reply = (method, result) => { const m = [...proc.sent].reverse().find((x) => x.method === method)!; say({ jsonrpc: '2.0', id: m.id, result }); };
    proc.stdin = {
      write: (t, cb) => {
        const m = JSON.parse(t);
        proc.sent.push(m);
        cb?.();
        if (m.method === 'initialize') queueMicrotask(() => say({ jsonrpc: '2.0', id: m.id, result: { protocolVersion: 1, agentCapabilities: { sessionCapabilities: { resume: {} }, _meta: { antigravity: { systemPrompt: { append: true } } } } } }));
        if (m.method === 'session/new') queueMicrotask(() => say({ jsonrpc: '2.0', id: m.id, result: { sessionId: 'sess-c', modes: { currentModeId: 'auto', availableModes: [{ id: 'auto' }] } } }));
        return true;
      },
      end: () => { proc.ended = true; },
    };
    proc.kill = () => { proc.killed = true; };
    return proc;
  }

  const supervisorWith = (relay: ReturnType<typeof fakeRelay>, asked: { args?: string[]; env?: NodeJS.ProcessEnv }) => Object.assign(Object.create(Supervisor.prototype), {
    appDir: '/app', userDir: '/u', config: { storeRoot: '/s', accountId: 'acct', antigravityBin: null },
    storeMcpCommand: () => null, _cloudDocsEnv: () => ({}),
    _codexApprovals: () => createCodexApprovals({ storeRoot: fs.mkdtempSync(path.join(os.tmpdir(), 'agy-cards-')) }),
    _spawnCloudRelay: (args: string[], { env }: { env: NodeJS.ProcessEnv }) => { asked.args = args; asked.env = env; return relay; },
  });

  it('starts agy in the cloud through the relay, ends its input after the turn, and ends when the work is home', async () => {
    const relay = fakeRelay();
    const asked: { args?: string[]; env?: NodeJS.ProcessEnv } = {};
    const worker = supervisorWith(relay, asked)._spawnAntigravityWorker({ args: [], prompt: 'Fix it', system: 'Rules' }, { cwd: '/w', item: { id: 'w-1', product: 'shop' }, env: { PATH: '/bin' }, cloud: true });
    let exit: unknown[] | null = null;
    worker.on('exit', (...a: unknown[]) => { exit = a; });
    expect(asked.args).toEqual(['--agentbox-agy', 'acp', '--disable-telemetry', '--gemini-dir', '/u/antigravity']);
    expect(asked.env).toMatchObject({ ZERO_ITEM: 'w-1', ZERO_PRODUCT: 'shop', STORE_ACCOUNT_ID: 'acct' });
    await vi.waitFor(() => expect(relay.sent.some((m) => m.method === 'session/prompt')).toBe(true));
    relay.reply('session/prompt', { stopReason: 'end_turn' });
    await vi.waitFor(() => expect(relay.ended).toBe(true));
    expect(relay.killed).toBe(false);
    expect(exit).toBeNull();
    relay.emit('exit', 0, null);
    await vi.waitFor(() => expect(exit?.[0]).toBe(0));
  });

  it('says the cloud sign-in needs doing again when the relay says so', async () => {
    const relay = fakeRelay();
    const worker = supervisorWith(relay, {})._spawnAntigravityWorker({ args: [], prompt: 'Fix it' }, { cwd: '/w', item: { id: 'w-1', product: 'shop' }, env: {}, cloud: true });
    let exit: unknown[] | null = null;
    worker.on('exit', (...a: unknown[]) => { exit = a; });
    await vi.waitFor(() => expect(relay.sent.some((m) => m.method === 'session/prompt')).toBe(true));
    relay.reply('session/prompt', { stopReason: 'end_turn' });
    relay.emit('exit', LOGIN_TROUBLE, null);
    await vi.waitFor(() => expect(exit).not.toBeNull());
    expect(exit![2]).toMatchObject({ cloudLoginTrouble: true });
  });
});

describe('an Antigravity run’s cloud side', () => {
  it('is asked for by its own flag and runs agy there', () => {
    expect(cloudEngineOf(['--agentbox-agy', 'acp', '--gemini-dir', '/h'])).toEqual({ engine: 'antigravity', remoteCommand: 'agy', argv: ['acp', '--gemini-dir', '/h'] });
    expect(cloudEngineOf(['--agentbox-codex', 'app-server'])).toEqual({ engine: 'codex', remoteCommand: 'codex', argv: ['app-server'] });
    expect(cloudEngineOf(['-p', 'x'])).toEqual({ engine: 'claude', remoteCommand: 'claude', argv: ['-p', 'x'] });
  });

  it('brings its conversations folder home, from the agy home the run was given', () => {
    expect(bringDirsFor('antigravity', { agyHome: '/h/agy', codexHome: '/c', projectsDir: '/p' })).toEqual([{ name: 'agy-conversations.tgz', dir: '/h/agy/antigravity-cli/conversations' }]);
    expect(bringDirsFor('codex', { agyHome: null, codexHome: '/c', projectsDir: '/p' })).toEqual([{ name: 'sessions.tgz', dir: '/c/sessions' }]);
    expect(bringDirsFor('claude', { agyHome: null, codexHome: '/c', projectsDir: '/p' })).toEqual([{ name: 'projects.tgz', dir: '/p' }]);
  });
});
