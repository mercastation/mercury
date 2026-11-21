// AN ANTIGRAVITY RUN WAITING ON ITS OWN COMMAND DOES NOT COME BACK TO YOU.
//
// What broke (2026-11-21, the sync thread w-f965f528bc): agy sent the 591 tests
// it had started to the background, ended its turn with "Aguardando o término
// dos 591 testes", and the worker read the answer to `session/prompt` as the
// end of the run. It closed agy, which stops what it started, so the tests
// died and the row came back to her as a promise.
//
// Measured on agy 1.5.13 (scripts/scratch/agy-background-*.mts): a
// `run_command` that outlives its WaitMsBeforeAsync (at most 10 s) becomes a
// background task, and agy's own prompt tells its model to say so and end the
// turn. Its `tool_call` stays `in_progress` past the prompt's answer; its
// `tool_call_update` (`completed`) arrives later with no prompt standing, and
// agy wakes its agent itself, whose work streams with no end marker: each step
// ends with a `usage_update`, the step's tool calls in the same millisecond. A
// command that ends while the turn is still going is answered inside it.
// `session/list` says nothing about it. So the run stays open while a tool call
// is still running, the agent is told once, by name, as a Claude Code run is
// (main/claude-input.mts), and the run ends once nothing is left running and
// the agent's newest step ended with no tool call.
import { describe, it, expect, vi, afterEach } from 'vitest';
import { EventEmitter } from 'node:events';
import { createAntigravityWorker, IDLE_SETTLE_MS, QUIET_END_MS } from '../main/antigravity-worker.mts';
import { LEFT_RUNNING_MS, MESSAGE_DONE, TURN_DONE, captureAntigravityEvent } from '../main/antigravity.mts';
import type { AntigravitySession } from '../main/antigravity.mts';
import { antigravityActivity, currentActivity } from '../main/agent-activity.mts';
import type { ActivitySession } from '../main/agent-activity.mts';

interface Msg { id?: number | string; method?: string; params?: Record<string, unknown> }

const SESSION = { sessionId: 'sess-1', modes: { currentModeId: 'auto', availableModes: [{ id: 'auto' }] }, configOptions: [] };
const CAPABILITIES = { loadSession: true, sessionCapabilities: { resume: {} }, _meta: { antigravity: { systemPrompt: { append: true } } } };

function fakeAgy() {
  const child = new EventEmitter() as EventEmitter & { stdout: EventEmitter; stderr: EventEmitter; sent: Msg[]; killed: boolean; pid: number; stdin: { write(t: string, cb?: () => void): boolean; end(): void }; kill(): void; say(m: unknown): void; prompts(): string[]; answer(result: unknown): void };
  child.stdout = new EventEmitter();
  child.stderr = new EventEmitter();
  child.sent = [];
  child.killed = false;
  child.pid = 4242;
  child.say = m => child.stdout.emit('data', Buffer.from(`${JSON.stringify(m)}\n`));
  const prompts = () => child.sent.filter(m => m.method === 'session/prompt');
  child.prompts = () => prompts().map(m => String((m.params!.prompt as { text?: string }[])[0].text));
  child.answer = result => child.say({ jsonrpc: '2.0', id: prompts().at(-1)!.id, result });
  child.stdin = {
    write: (t, cb) => {
      const m = JSON.parse(t) as Msg;
      child.sent.push(m);
      cb?.();
      if (m.method === 'initialize') queueMicrotask(() => child.say({ jsonrpc: '2.0', id: m.id, result: { protocolVersion: 1, agentCapabilities: CAPABILITIES } }));
      if (m.method === 'session/new') queueMicrotask(() => child.say({ jsonrpc: '2.0', id: m.id, result: SESSION }));
      return true;
    },
    end: () => {},
  };
  child.kill = () => { if (child.killed) return; child.killed = true; queueMicrotask(() => child.emit('exit', null, 'SIGTERM')); };
  return child;
}

const update = (u: Record<string, unknown>) => ({ jsonrpc: '2.0', method: 'session/update', params: { sessionId: 'sess-1', update: u } });
const says = (text: string) => update({ sessionUpdate: 'agent_message_chunk', content: { type: 'text', text } });
const stepEnds = () => update({ sessionUpdate: 'usage_update', used: 10_000, size: 1_048_576 });
const runs = (id: string, command: string, toolSummary?: string) => update({ sessionUpdate: 'tool_call', toolCallId: id, name: 'run_command', kind: 'execute', status: 'in_progress', title: command, rawInput: { CommandLine: command, WaitMsBeforeAsync: 1000, ...(toolSummary ? { toolSummary } : {}) } });
const ends = (id: string, command: string) => update({ sessionUpdate: 'tool_call_update', toolCallId: id, name: 'run_command', kind: 'execute', status: 'completed', title: command, rawInput: { CommandLine: command } });

function start(options: Partial<Parameters<typeof createAntigravityWorker>[0]> = {}) {
  const agy = fakeAgy();
  const events: [string, Record<string, unknown>][] = [];
  const worker = createAntigravityWorker({ spawnAgent: () => agy, cwd: '/tmp/task', prompt: 'Sync and run the tests', system: 'The rules', ...options });
  worker.on('event', (method: string, params: Record<string, unknown>) => events.push([method, params]));
  let code: number | null | undefined;
  const exited = new Promise<number | null>(resolve => worker.once('exit', (c: number | null) => { code = c; resolve(c); }));
  return { worker, agy, events, exited, ended: () => code !== undefined, done: () => events.find(([m]) => m === TURN_DONE)?.[1] };
}

// The turn that ends on a promise: the tests went to the background, and the agent says it waits.
async function turnEndsWaiting(run: ReturnType<typeof start>, id = 'c1', command = 'npx vitest run', summary = 'Run the 591 tests') {
  await vi.waitFor(() => expect(run.agy.prompts()).toHaveLength(1));
  run.agy.say(runs(id, command, summary));
  run.agy.say(says('Aguardando o término dos 591 testes.'));
  run.agy.say(stepEnds());
  run.agy.answer({ stopReason: 'end_turn' });
}

// Time for the worker to read what came, as agy's next words come seconds later.
const tick = () => (vi.isFakeTimers() ? vi.advanceTimersByTimeAsync(1) : new Promise(resolve => setTimeout(resolve, 20)));

// What the agent says back to the note, and the end of that turn.
async function noteAnswered(run: ReturnType<typeof start>, line = 'Waiting for the tests to finish.', prompts = 2) {
  await vi.waitFor(() => expect(run.agy.prompts()).toHaveLength(prompts));
  run.agy.say(says(line));
  run.agy.say(stepEnds());
  run.agy.answer({ stopReason: 'end_turn' });
  await tick();
}

afterEach(() => { vi.useRealTimers(); });

describe('an Antigravity run whose turn ends with a command still running', () => {
  it('stays open, and tells the agent once, by name, what it left running', async () => {
    const run = start();
    await turnEndsWaiting(run);
    await vi.waitFor(() => expect(run.agy.prompts()).toHaveLength(2));
    const note = run.agy.prompts()[1];
    expect(note).toMatch(/not the person/);
    expect(note).toMatch(/"Run the 591 tests"/);
    expect(note).toMatch(/[Ss]top any you no longer need/);
    expect(run.agy.killed).toBe(false);
    expect(run.ended()).toBe(false);
    expect(run.done()).toBeUndefined();
  });

  it('ends once the command has finished and the agent has answered, with that answer', async () => {
    vi.useFakeTimers();
    const run = start();
    await turnEndsWaiting(run);
    await noteAnswered(run);
    // However long the suite takes, short of the half hour.
    await vi.advanceTimersByTimeAsync(20 * 60_000);
    expect(run.ended()).toBe(false);
    expect(run.agy.killed).toBe(false);
    // agy wakes its agent itself when the command ends.
    run.agy.say(ends('c1', 'npx vitest run'));
    run.agy.say(says('591 passed. Shipped.'));
    run.agy.say(stepEnds());
    await vi.advanceTimersByTimeAsync(IDLE_SETTLE_MS / 2);
    expect(run.ended()).toBe(false);
    await vi.advanceTimersByTimeAsync(IDLE_SETTLE_MS);
    expect(await run.exited).toBe(0);
    expect(run.done()).toMatchObject({ sessionId: 'sess-1', stopReason: 'end_turn', text: '591 passed. Shipped.' });
    expect(run.events.filter(([m]) => m === MESSAGE_DONE).map(([, p]) => p.text)).toEqual(['Aguardando o término dos 591 testes.', 'Waiting for the tests to finish.', '591 passed. Shipped.']);
    expect(run.agy.prompts()).toHaveLength(2);
    expect(run.agy.killed).toBe(true);
  });

  it('does not end in the middle of what the agent does once it wakes', async () => {
    vi.useFakeTimers();
    const run = start();
    await turnEndsWaiting(run);
    await noteAnswered(run);
    run.agy.say(ends('c1', 'npx vitest run'));
    run.agy.say(stepEnds());
    run.agy.say(update({ sessionUpdate: 'tool_call', toolCallId: 'e1', name: 'edit_file', kind: 'edit', status: 'in_progress', title: 'File a.ts', rawInput: { Path: '/tmp/task/a.ts' } }));
    await vi.advanceTimersByTimeAsync(4 * IDLE_SETTLE_MS);
    expect(run.ended()).toBe(false);
    run.agy.say(update({ sessionUpdate: 'tool_call_update', toolCallId: 'e1', status: 'completed' }));
    run.agy.say(says('Fixed the one that failed.'));
    run.agy.say(stepEnds());
    await vi.advanceTimersByTimeAsync(IDLE_SETTLE_MS + 100);
    expect(await run.exited).toBe(0);
    expect(run.done()!.text).toBe('Fixed the one that failed.');
  });

  it('still ends at once when the command finished inside the turn, or nothing was started', async () => {
    const run = start();
    await vi.waitFor(() => expect(run.agy.prompts()).toHaveLength(1));
    run.agy.say(runs('c1', 'npm test'));
    run.agy.say(ends('c1', 'npm test'));
    run.agy.say(says('All pass.'));
    run.agy.say(stepEnds());
    run.agy.answer({ stopReason: 'end_turn' });
    expect(await run.exited).toBe(0);
    expect(run.agy.prompts()).toHaveLength(1);

    const plain = start();
    await vi.waitFor(() => expect(plain.agy.prompts()).toHaveLength(1));
    plain.agy.say(says('Done.'));
    plain.agy.answer({ stopReason: 'end_turn' });
    expect(await plain.exited).toBe(0);
    expect(plain.done()!.text).toBe('Done.');
  });

  it('takes only a call that outlived a step for one sent to the background', async () => {
    // A call that began after the turn's last step is not one agy wakes for.
    const run = start();
    await vi.waitFor(() => expect(run.agy.prompts()).toHaveLength(1));
    run.agy.say(says('Done.'));
    run.agy.say(stepEnds());
    run.agy.say(update({ sessionUpdate: 'tool_call', toolCallId: 't1', name: 'notify_user', kind: 'other', status: 'in_progress', title: 'Tell the user' }));
    run.agy.answer({ stopReason: 'end_turn' });
    expect(await run.exited).toBe(0);
    expect(run.agy.prompts()).toHaveLength(1);
  });

  it('waits for the wake when the command finished just as the turn ended', async () => {
    vi.useFakeTimers();
    const run = start();
    await vi.waitFor(() => expect(run.agy.prompts()).toHaveLength(1));
    run.agy.say(runs('c1', 'npm test'));
    run.agy.say(says('Waiting for npm test.'));
    run.agy.say(stepEnds());
    // Newer than the turn's last step: agy had stopped, so it wakes its agent for it.
    run.agy.say(ends('c1', 'npm test'));
    run.agy.answer({ stopReason: 'end_turn' });
    await vi.advanceTimersByTimeAsync(IDLE_SETTLE_MS * 3);
    expect(run.ended()).toBe(false);
    expect(run.agy.prompts()).toHaveLength(1);
    run.agy.say(says('All pass.'));
    run.agy.say(stepEnds());
    await vi.advanceTimersByTimeAsync(IDLE_SETTLE_MS + 100);
    expect(await run.exited).toBe(0);
    expect(run.done()!.text).toBe('All pass.');
  });

  it('waits for every command, and names again only one started after the first were named', async () => {
    vi.useFakeTimers();
    const run = start();
    await vi.waitFor(() => expect(run.agy.prompts()).toHaveLength(1));
    run.agy.say(runs('c1', 'npm run build', 'Build the app'));
    run.agy.say(runs('c2', 'npx vitest run', 'Run the tests'));
    run.agy.say(says('Both are running.'));
    run.agy.say(stepEnds());
    run.agy.answer({ stopReason: 'end_turn' });
    await noteAnswered(run);
    expect(run.agy.prompts()[1]).toMatch(/"Build the app", "Run the tests"/);
    // The build ends; the agent wakes, starts the screenshots, and waits again.
    run.agy.say(ends('c1', 'npm run build'));
    run.agy.say(stepEnds());
    run.agy.say(runs('c3', 'node shots.mjs', 'Take the screenshots'));
    run.agy.say(says('Build done; tests and screenshots still running.'));
    run.agy.say(stepEnds());
    await vi.advanceTimersByTimeAsync(IDLE_SETTLE_MS + 100);
    await vi.waitFor(() => expect(run.agy.prompts()).toHaveLength(3));
    expect(run.agy.prompts()[2]).toMatch(/"Run the tests", "Take the screenshots"/);
    await noteAnswered(run, 'Waiting on both.', 3);
    run.agy.say(ends('c2', 'npx vitest run'));
    run.agy.say(says('Tests pass; screenshots still going.'));
    run.agy.say(stepEnds());
    await vi.advanceTimersByTimeAsync(IDLE_SETTLE_MS * 3);
    expect(run.ended()).toBe(false);
    expect(run.agy.prompts()).toHaveLength(3);
    run.agy.say(ends('c3', 'node shots.mjs'));
    run.agy.say(says('All done.'));
    run.agy.say(stepEnds());
    await vi.advanceTimersByTimeAsync(IDLE_SETTLE_MS + 100);
    expect(await run.exited).toBe(0);
    expect(run.done()!.text).toBe('All done.');
  });

  it('a stop while it waits is a stop, so the promise is never taken for the answer', async () => {
    const run = start();
    await turnEndsWaiting(run);
    await noteAnswered(run);
    await new Promise(resolve => setTimeout(resolve, 20));
    run.worker.kill();
    expect(await run.exited).toBe(1);
    expect(run.done()).toMatchObject({ stopReason: 'cancelled' });
    expect(run.agy.killed).toBe(true);
    const session: AntigravitySession = {};
    for (const [m, p] of run.events) captureAntigravityEvent(session, m, p);
    expect(session.resultIsError).toBe(true);
  });

  it('says so when agy dies while it waits', async () => {
    const run = start();
    await turnEndsWaiting(run);
    await noteAnswered(run);
    await new Promise(resolve => setTimeout(resolve, 20));
    run.agy.emit('exit', 137, null);
    expect(await run.exited).toBe(1);
    expect(String(run.done()!.error)).toMatch(/exited|closed/);
  });

  it('stops what was left running after half an hour without a word, and says so on the row', async () => {
    vi.useFakeTimers();
    const run = start();
    await turnEndsWaiting(run, 'c1', 'npm run dev', 'Start the dev server');
    await noteAnswered(run, 'The server is up for you to look at.');
    await vi.advanceTimersByTimeAsync(LEFT_RUNNING_MS - 60_000);
    expect(run.ended()).toBe(false);
    await vi.advanceTimersByTimeAsync(2 * 60_000);
    expect(await run.exited).toBe(0);
    expect(run.agy.killed).toBe(true);
    expect(run.done()).toMatchObject({ stopReason: 'end_turn', stopped: [{ task_id: 'c1', description: 'Start the dev server' }] });
    const session: AntigravitySession = {};
    for (const [m, p] of run.events) captureAntigravityEvent(session, m, p);
    expect(session.resultIsError).toBe(false);
    expect(session.result).toMatch(/^\*\*Reply to carry on\.\*\*/);
    expect(session.result).toMatch(/"Start the dev server"/);
    expect(session.result).toMatch(/The server is up for you to look at\./);
  });

  it('holds that half hour while a question waits on the person', async () => {
    vi.useFakeTimers();
    const run = start({ onApproval: () => new Promise(() => {}) });
    await turnEndsWaiting(run);
    await noteAnswered(run);
    await vi.advanceTimersByTimeAsync(1000);
    run.agy.say(update({ sessionUpdate: 'tool_call', toolCallId: 'p1', name: 'run_command', kind: 'execute', status: 'pending', title: 'curl example.com' }));
    run.agy.say({ jsonrpc: '2.0', id: 900, method: 'session/request_permission', params: { sessionId: 'sess-1', toolCall: { toolCallId: 'p1' }, options: [{ optionId: 'allow', kind: 'allow_once' }, { optionId: 'deny', kind: 'reject_once' }] } });
    await vi.advanceTimersByTimeAsync(2 * LEFT_RUNNING_MS);
    expect(run.ended()).toBe(false);
  });

  it('ends anyway when nothing is left running and agy never wakes', async () => {
    vi.useFakeTimers();
    const run = start();
    await vi.waitFor(() => expect(run.agy.prompts()).toHaveLength(1));
    run.agy.say(runs('c1', 'npm test'));
    run.agy.say(says('Waiting for npm test.'));
    run.agy.say(stepEnds());
    run.agy.say(ends('c1', 'npm test'));
    run.agy.answer({ stopReason: 'end_turn' });
    await vi.advanceTimersByTimeAsync(QUIET_END_MS - 60_000);
    expect(run.ended()).toBe(false);
    await vi.advanceTimersByTimeAsync(2 * 60_000);
    expect(await run.exited).toBe(0);
  });

  it('keeps the running row saying what it waits on after the turn ends', async () => {
    const run = start();
    const row: ActivitySession = {};
    run.worker.on('event', (m: string, p: Record<string, unknown>) => antigravityActivity(row, m, p));
    await turnEndsWaiting(run);
    await noteAnswered(run);
    await new Promise(resolve => setTimeout(resolve, 20));
    expect(currentActivity(row).map(a => a.label)).toEqual(['Running the tests']);
    run.agy.say(ends('c1', 'npx vitest run'));
    expect(currentActivity(row)).toEqual([]);
  });
});
