// One Antigravity run: one `agy acp` process holding one conversation, shaped
// like the child process the supervisor's spawn path reads (`stderr`, `event`
// frames, one `exit`), as the Codex and OpenCode workers are.
import { EventEmitter } from 'node:events';
import fs from 'node:fs';
import path from 'node:path';
import { createAcpClient, rejectionOption, ACP_PROTOCOL_VERSION } from './acp.mts';
import { MESSAGE_DONE, TURN_DONE } from './antigravity.mts';
import { ACP_PERMISSION } from './codex-approvals.mts';

/** agy's sandboxed mode that asks before leaving the sandbox, as Codex's Auto does. */
export const WORKER_MODE = 'auto';
/** How long a cancelled turn has to wind down before its process is closed. */
export const CANCEL_GRACE_MS = 5_000;
const PICTURE_CAP = 20 * 1024 * 1024;
const MIME: Record<string, string> = { '.png': 'image/png', '.jpg': 'image/jpeg', '.jpeg': 'image/jpeg', '.gif': 'image/gif', '.webp': 'image/webp' };

export interface AcpMcpServer { name: string; command: string; args: string[]; env: { name: string; value: string }[] }

interface AgentProcess {
  pid?: number;
  stdin: { write(line: string, callback?: (error?: Error | null) => void): unknown; end(): unknown };
  stdout?: { on(event: 'data', listener: (chunk: Buffer) => void): unknown; on(event: 'end', listener: () => void): unknown } | null;
  stderr?: { on(event: 'data', listener: (chunk: Buffer | string) => void): unknown } | null;
  on?(event: string, listener: (...args: never[]) => void): unknown;
  once?(event: string, listener: (...args: never[]) => void): unknown;
  kill?(): unknown;
}

export interface AntigravityWorkerOptions {
  spawnAgent: () => AgentProcess;
  cwd: string;
  prompt: string;
  system?: string | null;
  mcpServers?: AcpMcpServer[];
  resumeSessionId?: string | null;
  model?: string | null;
  effort?: string | null;
  mode?: string | null;
  pictures?: readonly string[];
  onApproval?: ((method: string, params: unknown) => unknown) | null;
  /** A cloud relay: its input is ended after the turn, never killed, so it can bring the work home. */
  endsWithItsInput?: boolean;
}

export interface AntigravityWorker extends EventEmitter {
  stderr: EventEmitter;
  stdout: EventEmitter;
  sessionId: string | null;
  pid?: number;
  kill(): void;
  done: Promise<void>;
}

interface ConfigOption { id?: unknown; currentValue?: unknown; options?: ({ value?: unknown } | null)[] }
interface Opened {
  sessionId?: unknown;
  modes?: { currentModeId?: unknown; availableModes?: ({ id?: unknown } | null)[] } | null;
  models?: { availableModels?: ({ modelId?: unknown } | null)[] } | null;
  configOptions?: (ConfigOption | null)[] | null;
}

const text = (value: unknown) => (typeof value === 'string' ? value : '');
const valuesOf = (option: ConfigOption | undefined) => (option?.options ?? []).map(o => o?.value).filter((v): v is string => typeof v === 'string');

export function createAntigravityWorker(o: AntigravityWorkerOptions): AntigravityWorker {
  const worker = new EventEmitter() as AntigravityWorker;
  worker.stderr = new EventEmitter();
  worker.stdout = new EventEmitter();
  worker.sessionId = o.resumeSessionId ?? null;
  const say = (line: unknown) => { const s = String(line ?? '').trim(); if (s) worker.stderr.emit('data', s); };

  let child: AgentProcess;
  try {
    child = o.spawnAgent();
  } catch (error) {
    // After this tick, so the supervisor's listeners are on before it ends.
    worker.kill = () => {};
    worker.done = Promise.resolve();
    setImmediate(() => { say((error as Error)?.message ?? error); worker.emit('exit', 1, null); });
    return worker;
  }
  worker.pid = child.pid;
  child.once?.('spawn', () => { worker.pid = child.pid; worker.emit('spawn'); });
  child.stderr?.on('data', chunk => worker.stderr.emit('data', chunk));

  let talking = '';
  let lastSaid = '';
  let replaying = false;
  let inTurn = false;
  let stopping = false;
  let ended = false;

  const flush = () => {
    const said = talking.trim();
    talking = '';
    if (!said) return;
    lastSaid = said;
    worker.emit('event', MESSAGE_DONE, { sessionId: worker.sessionId, text: said });
  };

  const client = createAcpClient({
    transport: child,
    clientInfo: { name: 'mercury', version: '1.0.0' },
    permissionTimeoutMs: 0,
    onNotification: (method, params) => {
      if (replaying) return;
      const u = (params as { update?: { sessionUpdate?: unknown; content?: { text?: unknown } } } | null)?.update;
      if (method === 'session/update' && u?.sessionUpdate === 'agent_message_chunk') talking += text(u.content?.text);
      else if (method === 'session/update' && (u?.sessionUpdate === 'tool_call' || u?.sessionUpdate === 'tool_call_update')) flush();
      worker.emit('event', method, params);
    },
    onPermission: async (request) => {
      const options = Array.isArray(request.options) ? request.options : [];
      const allow = options.find(option => option?.kind === 'allow_once')?.optionId ?? null;
      const deny = rejectionOption(options);
      if (stopping || !o.onApproval) return deny;
      let answer: unknown = null;
      try {
        answer = await o.onApproval(ACP_PERMISSION, request);
      } catch (error) {
        say(`Antigravity could not ask about a tool call: ${(error as Error)?.message ?? error}`);
      }
      return answer === 'accept' && !stopping ? allow : deny;
    },
  });

  const pictureBlocks = () => {
    const blocks: { type: 'image'; mimeType: string; data: string }[] = [];
    for (const file of o.pictures ?? []) {
      const mimeType = MIME[path.extname(file).toLowerCase()];
      try {
        if (!mimeType) throw Error('not a picture Antigravity reads');
        if (fs.statSync(file).size > PICTURE_CAP) throw Error('larger than 20 MB');
        blocks.push({ type: 'image', mimeType, data: fs.readFileSync(file).toString('base64') });
      } catch (error) {
        say(`A picture was left off this turn (${path.basename(file)}): ${(error as Error).message}`);
      }
    }
    return blocks;
  };

  const chooseMode = async (sessionId: string, opened: Opened | null) => {
    const want = o.mode || WORKER_MODE;
    const modes = opened?.modes;
    if (modes && !modes.availableModes?.some(m => m?.id === want)) {
      if (o.mode) throw Error(`Antigravity has no "${want}" mode.`);
      return;
    }
    if (modes?.currentModeId === want) return;
    await client.setMode({ sessionId, modeId: want });
  };

  // A row may name agy's base model ("gemini-4-neon") or a model with its level
  // ("gemini-4-neon-high"); the level the row names outranks the one in the id.
  const chooseOptions = async (sessionId: string, opened: Opened | null) => {
    let options = opened?.configOptions ?? [];
    const option = (id: string) => options.find(c => c?.id === id) ?? undefined;
    let effort = o.effort || null;
    if (o.model) {
      const models = valuesOf(option('model'));
      let base = models.includes(o.model) ? o.model : null;
      if (!base) {
        const levels = valuesOf(option('thought_level'));
        const split = models.find(m => o.model!.startsWith(`${m}-`) && levels.includes(o.model!.slice(m.length + 1)));
        if (split) { base = split; effort ??= o.model.slice(split.length + 1); }
      }
      if (base) {
        if (option('model')?.currentValue !== base) {
          const changed = await client.setConfigOption({ sessionId, configId: 'model', value: base }) as Opened | null;
          if (Array.isArray(changed?.configOptions)) options = changed.configOptions;
        }
      } else if (opened?.models?.availableModels?.some(m => m?.modelId === o.model)) {
        await client.setModel({ sessionId, modelId: o.model });
      } else {
        throw Error(`Antigravity has no model called "${o.model}" on this account. It offers: ${models.join(', ') || 'none'}.`);
      }
    }
    if (effort) {
      const levels = valuesOf(option('thought_level'));
      if (!levels.length) { say(`This Antigravity model has no thinking levels, so "${effort}" was set aside.`); return; }
      if (!levels.includes(effort)) throw Error(`Antigravity has no "${effort}" thinking level for this model. It offers: ${levels.join(', ')}.`);
      if (option('thought_level')?.currentValue !== effort) await client.setConfigOption({ sessionId, configId: 'thought_level', value: effort });
    }
  };

  const run = async () => {
    let code = 1;
    let startedAt = 0;
    try {
      const hello = await client.initialize() as { protocolVersion?: unknown } | null;
      if (hello?.protocolVersion !== ACP_PROTOCOL_VERSION) throw Error(`Antigravity answered in protocol version ${String(hello?.protocolVersion)}, and this app speaks ${ACP_PROTOCOL_VERSION}.`);
      const appends = (client.agentMeta()?.antigravity as { systemPrompt?: { append?: unknown } } | undefined)?.systemPrompt?.append === true;
      if (o.system && !appends) throw Error('This Antigravity cannot take standing instructions, so it would work against rules it was never given. Update agy and try again.');
      const _meta = o.system ? { antigravity: { systemPrompt: { append: o.system } } } : undefined;
      const where = { cwd: o.cwd, mcpServers: o.mcpServers ?? [], ...(_meta ? { _meta } : {}) };
      let opened: Opened | null;
      if (worker.sessionId) {
        if (client.can('resume')) opened = await client.resumeSession({ sessionId: worker.sessionId, ...where }) as Opened | null;
        else {
          replaying = true;
          try { opened = await client.loadSession({ sessionId: worker.sessionId, ...where }) as Opened | null; } finally { replaying = false; }
        }
        worker.emit('event', 'session/resume', { sessionId: worker.sessionId });
      } else {
        opened = await client.newSession(where) as Opened | null;
        worker.sessionId = text(opened?.sessionId) || null;
        if (!worker.sessionId) throw Error('Antigravity did not name the conversation it opened.');
        worker.emit('event', 'session/new', { sessionId: worker.sessionId });
      }
      const sessionId = worker.sessionId;
      await chooseMode(sessionId, opened);
      await chooseOptions(sessionId, opened);
      if (stopping) throw Error('The run was stopped before it began.');
      const pictures = o.pictures?.length ? (client.can('image') ? pictureBlocks() : []) : [];
      if (o.pictures?.length && !client.can('image')) say('This Antigravity takes no pictures, so the ones attached were left off.');
      startedAt = Date.now();
      inTurn = true;
      const answer = await client.prompt({ sessionId, blocks: [{ type: 'text', text: o.prompt }, ...pictures] }) as { stopReason?: unknown; usage?: unknown } | null;
      inTurn = false;
      flush();
      worker.emit('event', TURN_DONE, { sessionId, stopReason: answer?.stopReason ?? null, usage: answer?.usage ?? null, text: lastSaid, durationMs: Date.now() - startedAt });
      code = answer?.stopReason === 'end_turn' ? 0 : 1;
    } catch (error) {
      const why = (error as Error)?.message ?? String(error);
      if (inTurn) {
        inTurn = false;
        flush();
        worker.emit('event', TURN_DONE, { sessionId: worker.sessionId, error: why, text: lastSaid, durationMs: Date.now() - startedAt });
      }
      say(why);
      code = 1;
    } finally {
      ended = true;
      if (o.endsWithItsInput && !stopping) { try { child.stdin.end(); } catch { /* already closed */ } }
      else client.close('The run is over.');
      worker.emit('exit', code, null);
    }
  };

  worker.kill = () => {
    if (ended || stopping) return;
    stopping = true;
    if (inTurn && worker.sessionId) {
      client.cancel(worker.sessionId);
      setTimeout(() => client.close('The run was stopped.'), CANCEL_GRACE_MS).unref?.();
    } else {
      client.close('The run was stopped.');
    }
  };
  worker.done = run();
  return worker;
}
