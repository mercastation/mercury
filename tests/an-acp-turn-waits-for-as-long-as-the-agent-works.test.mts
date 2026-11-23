// 2026-11-19, first real agent on the ACP transport (Antigravity's `agy acp`
// 1.5.13). `session/prompt` answers only when the whole turn is over, and the
// transport gave every request two minutes, so a real task would have been cut
// off mid-turn. A permission question also expired after fifteen minutes with
// a deny, which the inbox stopped doing for its own cards (w-4b00022acb).
// Measured against the real agent: `_meta.antigravity.systemPrompt.append` on
// `session/new` reached the model, and `session/resume` reopened a
// conversation in a new process without replaying it.
import { describe, it, expect, vi, afterEach } from 'vitest';
import { EventEmitter } from 'node:events';
import { createAcpClient } from '../main/acp.mts';

interface Sent { id?: number; method?: string; params?: Record<string, unknown>; result?: unknown }

function fakeAgent() {
  const child = new EventEmitter() as EventEmitter & {
    stdout: EventEmitter; stderr: EventEmitter; sent: Sent[];
    stdin: { writable: boolean; write(text: string, cb?: () => void): boolean; end(): void; on(): void };
    kill(): void; say(message: unknown): void; reply(id: number, result: unknown): void; lastCall(method: string): Sent | null;
  };
  child.stdout = new EventEmitter();
  child.stderr = new EventEmitter();
  child.sent = [];
  child.stdin = { writable: true, write: (text, cb) => { child.sent.push(JSON.parse(text)); cb?.(); return true; }, end: () => { child.stdin.writable = false; }, on: () => {} };
  child.kill = () => {};
  child.say = message => child.stdout.emit('data', Buffer.from(`${JSON.stringify(message)}\n`));
  child.reply = (id, result) => child.say({ jsonrpc: '2.0', id, result });
  child.lastCall = method => [...child.sent].reverse().find(m => m.method === method) ?? null;
  return child;
}

async function ready(child: ReturnType<typeof fakeAgent>, capabilities: Record<string, unknown>) {
  const client = createAcpClient({ transport: child, requestTimeoutMs: 1_000 });
  const init = client.initialize();
  await vi.waitFor(() => expect(child.lastCall('initialize')).toBeTruthy());
  child.reply(child.lastCall('initialize')!.id!, { protocolVersion: 1, agentCapabilities: capabilities });
  await init;
  return client;
}

afterEach(() => { vi.useRealTimers(); });

describe('a turn on an ACP agent', () => {
  it('waits for a prompt far longer than an ordinary request may take', async () => {
    const child = fakeAgent();
    const client = await ready(child, {});
    vi.useFakeTimers();
    let answer: unknown = null;
    const turn = client.prompt({ sessionId: 's-1', text: 'a long task' }).then(r => { answer = r; });
    await vi.advanceTimersByTimeAsync(60 * 60_000);
    expect(answer).toBeNull();
    child.reply(child.lastCall('session/prompt')!.id!, { stopReason: 'end_turn' });
    await turn;
    expect(answer).toEqual({ stopReason: 'end_turn' });
  });

  it('still gives up on an ordinary request that never answers', async () => {
    const child = fakeAgent();
    const client = await ready(child, {});
    vi.useFakeTimers();
    const asked = client.setMode({ sessionId: 's-1', modeId: 'auto' });
    const outcome = expect(asked).rejects.toThrow(/did not answer/);
    await vi.advanceTimersByTimeAsync(1_001);
    await outcome;
  });

  // Since the 2026-11-23 sync Agentbox's transport has no question timer at
  // all, so this holds without being asked for.
  it('lets a permission question wait for its person', async () => {
    const child = fakeAgent();
    const seen: unknown[] = [];
    const client = createAcpClient({ transport: child, onPermission: req => { seen.push(req); return new Promise(() => {}); } });
    vi.useFakeTimers();
    child.say({ jsonrpc: '2.0', id: 9, method: 'session/request_permission', params: { sessionId: 's-1', options: [{ optionId: 'deny', kind: 'reject_once' }] } });
    await vi.advanceTimersByTimeAsync(24 * 60 * 60_000);
    expect(seen).toHaveLength(1);
    expect(child.sent.find(m => m.id === 9)).toBeUndefined();
    expect(client.openPermissions()).toBe(1);
  });

  it('carries the agent’s own extension on a new session only when one is given', async () => {
    const child = fakeAgent();
    const client = await ready(child, {});
    void client.newSession({ cwd: '/task', mcpServers: [] });
    expect(child.lastCall('session/new')!.params).toEqual({ cwd: '/task', mcpServers: [] });
    const _meta = { antigravity: { systemPrompt: { append: 'rules' } } };
    void client.newSession({ cwd: '/task', mcpServers: [], _meta });
    expect(child.lastCall('session/new')!.params).toEqual({ cwd: '/task', mcpServers: [], _meta });
  });

  it('reopens a conversation without replay only on an agent that offered it', async () => {
    const without = fakeAgent();
    const plain = await ready(without, { loadSession: true });
    expect(plain.can('resume')).toBe(false);
    await expect(plain.resumeSession({ sessionId: 's-1', cwd: '/task' })).rejects.toThrow(/without replaying/);
    expect(without.lastCall('session/resume')).toBeNull();

    const child = fakeAgent();
    const client = await ready(child, { loadSession: true, sessionCapabilities: { resume: {}, load: {} } });
    expect(client.can('resume')).toBe(true);
    void client.resumeSession({ sessionId: 's-1', cwd: '/task', mcpServers: [] });
    expect(child.lastCall('session/resume')!.params).toEqual({ sessionId: 's-1', cwd: '/task', mcpServers: [] });
  });

  it('sets a mode and a session option in the protocol’s own words', async () => {
    const child = fakeAgent();
    const client = await ready(child, {});
    void client.setMode({ sessionId: 's-1', modeId: 'auto' });
    expect(child.lastCall('session/set_mode')!.params).toEqual({ sessionId: 's-1', modeId: 'auto' });
    void client.setConfigOption({ sessionId: 's-1', configId: 'thought_level', value: 'high' });
    expect(child.lastCall('session/set_config_option')!.params).toEqual({ sessionId: 's-1', configId: 'thought_level', value: 'high' });
  });
});
