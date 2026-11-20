// 2026-11-19: one Antigravity run is one `agy acp` process holding one
// conversation, measured on agy 1.5.13: `session/new` answered with the
// models, a `thought_level` option (low..xhigh) and four modes, starting in
// "default", which asks before every edit and command. Workers run in "auto",
// agy's sandboxed mode that asks before leaving the sandbox, the equivalent of
// Codex's Auto. A model or level the agent does not offer stops the run before
// the prompt, as a Codex model refusal does, rather than running on another.
import { describe, it, expect, vi } from 'vitest';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { EventEmitter } from 'node:events';
import { createAntigravityWorker } from '../main/antigravity-worker.mts';
import { MESSAGE_DONE, TURN_DONE } from '../main/antigravity.mts';

interface Msg { id?: number | string; method?: string; params?: Record<string, unknown>; result?: { outcome?: Record<string, unknown> } }

const NEW_SESSION = {
  sessionId: 'sess-1',
  modes: { currentModeId: 'default', availableModes: [{ id: 'default' }, { id: 'accept-edits' }, { id: 'auto' }, { id: 'plan' }] },
  configOptions: [
    { id: 'model', type: 'select', currentValue: 'gemini-4-neon', options: [{ value: 'gemini-4-neon' }, { value: 'gemini-4-argon' }] },
    { id: 'thought_level', type: 'select', currentValue: 'high', options: [{ value: 'low' }, { value: 'medium' }, { value: 'high' }, { value: 'xhigh' }] },
  ],
};
const CAPABILITIES = { loadSession: true, promptCapabilities: { image: true }, sessionCapabilities: { resume: {} }, _meta: { antigravity: { systemPrompt: { append: true } } } };

function fakeAgy({ capabilities = CAPABILITIES, session = NEW_SESSION }: { capabilities?: Record<string, unknown>; session?: Record<string, unknown> } = {}) {
  const child = new EventEmitter() as EventEmitter & { stdout: EventEmitter; stderr: EventEmitter; sent: Msg[]; killed: boolean; pid: number; stdin: { write(t: string, cb?: () => void): boolean; end(): void }; kill(): void; say(m: unknown): void; lastCall(method: string): Msg | null; answer(method: string, result: unknown): void };
  child.stdout = new EventEmitter();
  child.stderr = new EventEmitter();
  child.sent = [];
  child.killed = false;
  child.pid = 4242;
  child.say = m => child.stdout.emit('data', Buffer.from(`${JSON.stringify(m)}\n`));
  child.lastCall = method => [...child.sent].reverse().find(m => m.method === method) ?? null;
  child.answer = (method, result) => { const call = child.lastCall(method); child.say({ jsonrpc: '2.0', id: call!.id, result }); };
  child.stdin = {
    write: (t, cb) => {
      const m = JSON.parse(t) as Msg;
      child.sent.push(m);
      cb?.();
      if (m.method === 'initialize') queueMicrotask(() => child.say({ jsonrpc: '2.0', id: m.id, result: { protocolVersion: 1, agentCapabilities: capabilities } }));
      if (m.method === 'session/new' || m.method === 'session/resume') queueMicrotask(() => child.say({ jsonrpc: '2.0', id: m.id, result: m.method === 'session/new' ? session : { ...session, sessionId: undefined } }));
      if (m.method === 'session/set_mode' || m.method === 'session/set_config_option') queueMicrotask(() => child.say({ jsonrpc: '2.0', id: m.id, result: {} }));
      return true;
    },
    end: () => {},
  };
  child.kill = () => { if (child.killed) return; child.killed = true; queueMicrotask(() => child.emit('exit', null, 'SIGTERM')); };
  return child;
}

const chunk = (text: string) => ({ jsonrpc: '2.0', method: 'session/update', params: { sessionId: 'sess-1', update: { sessionUpdate: 'agent_message_chunk', content: { type: 'text', text } } } });
const tool = (status: string) => ({ jsonrpc: '2.0', method: 'session/update', params: { sessionId: 'sess-1', update: { sessionUpdate: status === 'in_progress' ? 'tool_call' : 'tool_call_update', toolCallId: 'c1', kind: 'execute', status, rawInput: { CommandLine: 'npm test' } } } });

function start(options: Partial<Parameters<typeof createAntigravityWorker>[0]> = {}, agy = fakeAgy()) {
  const events: [string, Record<string, unknown>][] = [];
  const worker = createAntigravityWorker({ spawnAgent: () => agy, cwd: '/tmp/task', prompt: 'Fix the tests', system: 'The rules', mcpServers: [{ name: 'mercury', command: '/bin/store', args: [], env: [] }], ...options });
  worker.on('event', (method: string, params: Record<string, unknown>) => events.push([method, params]));
  const stderr: string[] = [];
  worker.stderr.on('data', (d: unknown) => stderr.push(String(d)));
  const exited = new Promise<number | null>(resolve => worker.once('exit', (code: number | null) => resolve(code)));
  return { worker, agy, events, stderr, exited };
}

describe('an Antigravity worker', () => {
  it('opens a conversation in the task folder with the standing rules, the store and Auto, then carries one turn', async () => {
    const { agy, events, exited, worker } = start();
    await vi.waitFor(() => expect(agy.lastCall('session/prompt')).toBeTruthy());
    expect(agy.lastCall('session/new')!.params).toEqual({ cwd: '/tmp/task', mcpServers: [{ name: 'mercury', command: '/bin/store', args: [], env: [] }], _meta: { antigravity: { systemPrompt: { append: 'The rules' } } } });
    expect(agy.lastCall('session/set_mode')!.params).toEqual({ sessionId: 'sess-1', modeId: 'auto' });
    expect(agy.lastCall('session/prompt')!.params).toEqual({ sessionId: 'sess-1', prompt: [{ type: 'text', text: 'Fix the tests' }] });
    expect(worker.sessionId).toBe('sess-1');
    expect(events[0]).toEqual(['session/new', { sessionId: 'sess-1' }]);
    agy.say(chunk('Looking at '));
    agy.say(chunk('the tests.'));
    agy.say(tool('in_progress'));
    agy.say(tool('completed'));
    agy.say(chunk('All pass.'));
    agy.answer('session/prompt', { stopReason: 'end_turn', usage: { totalTokens: 10 } });
    expect(await exited).toBe(0);
    const marks = events.filter(([m]) => m === MESSAGE_DONE || m === TURN_DONE);
    expect(marks.map(([m, p]) => [m, p.text])).toEqual([[MESSAGE_DONE, 'Looking at the tests.'], [MESSAGE_DONE, 'All pass.'], [TURN_DONE, 'All pass.']]);
    expect(marks[2][1]).toMatchObject({ sessionId: 'sess-1', stopReason: 'end_turn', usage: { totalTokens: 10 } });
    expect(agy.killed).toBe(true);
  });

  it('picks a conversation back up without replaying it, and never starts a new one in its place', async () => {
    const { agy, exited } = start({ resumeSessionId: 'sess-old' });
    await vi.waitFor(() => expect(agy.lastCall('session/prompt')).toBeTruthy());
    expect(agy.lastCall('session/new')).toBeNull();
    expect(agy.lastCall('session/resume')!.params).toMatchObject({ sessionId: 'sess-old', cwd: '/tmp/task' });
    expect(agy.lastCall('session/prompt')!.params!.sessionId).toBe('sess-old');
    agy.answer('session/prompt', { stopReason: 'end_turn' });
    expect(await exited).toBe(0);
  });

  it('will not run where the standing rules cannot reach', async () => {
    const { agy, stderr, exited } = start({}, fakeAgy({ capabilities: { loadSession: true } }));
    expect(await exited).toBe(1);
    expect(agy.lastCall('session/prompt')).toBeNull();
    expect(stderr.join('')).toMatch(/instructions/i);
  });

  it('runs on the model and level the thread names, and refuses ones the agent does not have', async () => {
    const ok = start({ model: 'gemini-4-argon', effort: 'xhigh' });
    await vi.waitFor(() => expect(ok.agy.lastCall('session/prompt')).toBeTruthy());
    const set = ok.agy.sent.filter(m => m.method === 'session/set_config_option').map(m => m.params);
    expect(set).toEqual([{ sessionId: 'sess-1', configId: 'model', value: 'gemini-4-argon' }, { sessionId: 'sess-1', configId: 'thought_level', value: 'xhigh' }]);

    const model = start({ model: 'gpt-5.6-sol' });
    expect(await model.exited).toBe(1);
    expect(model.agy.lastCall('session/prompt')).toBeNull();
    expect(model.stderr.join('')).toMatch(/gpt-5\.6-sol/);

    const level = start({ effort: 'max' });
    expect(await level.exited).toBe(1);
    expect(level.stderr.join('')).toMatch(/max/);
  });

  it('sets a level aside, saying so, on a model that has none to choose', async () => {
    // Measured: after switching to `gemini-pro-orchestrator-neon` agy's options carry no thought_level at all.
    const session = { ...NEW_SESSION, configOptions: [{ id: 'model', type: 'select', currentValue: 'gemini-pro-orchestrator-neon', options: [{ value: 'gemini-pro-orchestrator-neon' }] }] };
    const { agy, stderr } = start({ effort: 'high' }, fakeAgy({ session }));
    await vi.waitFor(() => expect(agy.lastCall('session/prompt')).toBeTruthy());
    expect(agy.lastCall('session/set_config_option')).toBeNull();
    expect(stderr.join('')).toMatch(/no thinking levels/i);
  });

  it('attaches the pictures sent with the thread as pictures', async () => {
    const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'agy-pic-'));
    const file = path.join(dir, 'shot.png');
    fs.writeFileSync(file, Buffer.from([0x89, 0x50, 0x4e, 0x47]));
    const { agy } = start({ pictures: [file] });
    await vi.waitFor(() => expect(agy.lastCall('session/prompt')).toBeTruthy());
    expect(agy.lastCall('session/prompt')!.params!.prompt).toEqual([
      { type: 'text', text: 'Fix the tests' },
      { type: 'image', mimeType: 'image/png', data: Buffer.from([0x89, 0x50, 0x4e, 0x47]).toString('base64') },
    ]);
    fs.rmSync(dir, { recursive: true, force: true });
  });

  it('answers a question with the option the card chose, and refuses when the card cannot be asked', async () => {
    const answers = ['accept', 'decline'];
    const asked: unknown[] = [];
    const { agy } = start({ onApproval: async (_m: string, p: unknown) => { asked.push(p); const a = answers.shift(); if (!a) throw Error('spool gone'); return a; } });
    await vi.waitFor(() => expect(agy.lastCall('session/prompt')).toBeTruthy());
    const ask = (id: number) => agy.say({ jsonrpc: '2.0', id, method: 'session/request_permission', params: { sessionId: 'sess-1', toolCall: { toolCallId: 'c1' }, options: [{ optionId: 'allow', kind: 'allow_once' }, { optionId: 'deny', kind: 'reject_once' }] } });
    for (const id of [101, 102, 103]) ask(id);
    await vi.waitFor(() => expect(agy.sent.filter(m => m.result).length).toBe(3));
    const outcome = (id: number) => agy.sent.find(m => m.id === id)!.result!.outcome;
    expect(outcome(101)).toEqual({ outcome: 'selected', optionId: 'allow' });
    expect(outcome(102)).toEqual({ outcome: 'selected', optionId: 'deny' });
    expect(outcome(103)).toEqual({ outcome: 'selected', optionId: 'deny' });
    expect(asked).toHaveLength(3);
  });

  it('cancels the turn when stopped, then lets the process go', async () => {
    const { agy, worker, exited, events } = start();
    await vi.waitFor(() => expect(agy.lastCall('session/prompt')).toBeTruthy());
    worker.kill();
    await vi.waitFor(() => expect(agy.lastCall('session/cancel')).toBeTruthy());
    agy.answer('session/prompt', { stopReason: 'cancelled' });
    expect(await exited).toBe(1);
    expect(events.find(([m]) => m === TURN_DONE)![1]).toMatchObject({ stopReason: 'cancelled' });
    expect(agy.killed).toBe(true);
  });

  it('says so when the agent dies in the middle of a turn', async () => {
    const { agy, exited, events } = start();
    await vi.waitFor(() => expect(agy.lastCall('session/prompt')).toBeTruthy());
    agy.emit('exit', 137, null);
    expect(await exited).toBe(1);
    expect(String(events.find(([m]) => m === TURN_DONE)![1].error)).toMatch(/exited|closed/);
  });
});
