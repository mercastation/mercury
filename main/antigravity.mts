// Reading an Antigravity run (`agy acp`) in the protocol's own words: the
// `session/update` notifications pass through untranslated. Two frames are the
// worker's own, because ACP has no "message finished" event and the stateless
// trace and summary readers need whole messages: `turn/message` when a run of
// prose ends, and `session/prompt`, the prompt request's own answer, when the
// turn ends. Shapes read off agy 1.5.13 on 2026-11-19.
import { SAYING_CAP } from './codex.mts';
import { nameSlugs } from '../shared/product-name.mjs';

export const MESSAGE_DONE = 'turn/message';
export const TURN_DONE = 'session/prompt';

/** The store server's names, which every run is given (main/supervisor.mts STORE_SERVER). */
export const STORE_SERVERS: readonly string[] = nameSlugs;

export interface AcpLocation { path?: unknown }

export interface AcpUpdate {
  sessionUpdate?: unknown;
  content?: unknown;
  toolCallId?: unknown;
  name?: unknown;
  kind?: unknown;
  status?: unknown;
  title?: unknown;
  rawInput?: Record<string, unknown> | null;
  locations?: (AcpLocation | null)[] | null;
}

export interface AcpFrame {
  sessionId?: unknown;
  update?: AcpUpdate | null;
  text?: unknown;
  stopReason?: unknown;
  error?: unknown;
  durationMs?: unknown;
  usage?: unknown;
}

export interface AntigravitySession {
  sessionId?: string | null;
  result?: string;
  resultIsError?: boolean;
  saying?: string;
  sayingAt?: number;
  antigravityTalking?: boolean;
  antigravityUsage?: unknown;
  antigravityChange?: string[];
}

const frameOf = (params: unknown): AcpFrame => (params && typeof params === 'object' ? params as AcpFrame : {});
const text = (value: unknown) => (typeof value === 'string' ? value : '');
const chunkText = (u: AcpUpdate | null | undefined) => text((u?.content as { text?: unknown } | null | undefined)?.text);

const TROUBLE: Record<string, string> = {
  cancelled: 'The turn was cancelled before it finished.',
  refusal: 'Antigravity refused to carry on with this turn.',
  max_tokens: 'The answer ran past the model’s output limit.',
  max_turn_requests: 'The turn reached Antigravity’s limit on steps.',
};

/** Why a turn did not end on its own, or null when it did. */
export function turnTrouble(frame: AcpFrame): string | null {
  const error = text(frame.error).trim();
  if (error) return error;
  const reason = text(frame.stopReason);
  if (!reason || reason === 'end_turn') return null;
  return TROUBLE[reason] ?? `The turn ended early (${reason}).`;
}

export function captureAntigravityEvent(session: AntigravitySession | null | undefined, method: string, params: unknown): void {
  try {
    if (!session || typeof method !== 'string') return;
    const p = frameOf(params);
    if (!session.sessionId && typeof p.sessionId === 'string' && p.sessionId) session.sessionId = p.sessionId;
    if (method === MESSAGE_DONE) {
      const said = text(p.text).trim();
      if (said) { session.result = said; session.resultIsError = false; }
      return;
    }
    if (method === TURN_DONE) {
      if (p.usage) session.antigravityUsage = p.usage;
      const trouble = turnTrouble(p);
      if (trouble) { session.result = trouble; session.resultIsError = true; }
    }
  } catch { /* a reader that throws kills a run that was going fine */ }
}

// Each message starts from empty, so the live text matches the traced block;
// it is not cleared when the message ends, or it would blink off screen.
export function antigravityStreamingText(session: AntigravitySession | null | undefined, method: string, params: unknown): boolean {
  try {
    if (!session) return false;
    if (method === MESSAGE_DONE) { session.antigravityTalking = false; return false; }
    if (method !== 'session/update') return false;
    const u = frameOf(params).update;
    if (u?.sessionUpdate !== 'agent_message_chunk') return false;
    const piece = chunkText(u);
    if (!piece) return false;
    if (!session.antigravityTalking) {
      session.antigravityTalking = true;
      session.saying = '';
      session.sayingAt = Date.now();
    }
    const next = (session.saying ?? '') + piece;
    session.saying = next.length > SAYING_CAP ? '' : next;
    return true;
  } catch {
    return false;
  }
}

/**
 * A tool name as the rest of the app spells one. agy writes an MCP tool as
 * `mcp_<server>_<tool>`, and a server name may hold underscores, so only the
 * servers this run was given are split; Claude Code's `mcp__<server>__<tool>`
 * is what the work lines and the bookkeeping filter read.
 */
export function antigravityToolName(name: unknown, servers: readonly string[] = []): string {
  const n = text(name).trim();
  if (!n.startsWith('mcp_')) return n;
  const server = [...servers].sort((a, b) => b.length - a.length).find(s => n.startsWith(`mcp_${s}_`));
  return server ? `mcp__${server}__${n.slice(`mcp_${server}_`.length)}` : n;
}

/** The tool word the run page and the work lines read, and what it acted on. */
export function toolWordOf(u: AcpUpdate | null | undefined, servers: readonly string[] = []): { word: string; detail: string } | null {
  if (!u) return null;
  const input = u.rawInput ?? {};
  const name = antigravityToolName(u.name, servers);
  const where = text(u.locations?.[0]?.path) || text(input.Path) || text(input.AbsolutePath) || text(input.TargetFile);
  switch (u.kind) {
    case 'execute': return { word: 'Bash', detail: text(input.CommandLine) || text(u.title) };
    case 'edit': return { word: /write|create/i.test(name) ? 'Write' : 'Edit', detail: where };
    case 'delete': return { word: 'Delete', detail: where };
    case 'move': return { word: 'Move', detail: where };
    case 'read': return { word: 'Read', detail: where };
    case 'search': return { word: 'Grep', detail: text(input.Query) || text(input.Pattern) || text(u.title) };
    case 'fetch': return { word: /search/i.test(name) ? 'WebSearch' : 'WebFetch', detail: text(input.Url) || text(input.Query) || text(u.title) };
    default: return name ? { word: name.replace(/\s+/g, '_'), detail: '' } : null;
  }
}

const finished = (status: unknown) => status === 'completed' || status === 'failed';
const toolUpdate = (u: AcpUpdate | null | undefined) => u?.sessionUpdate === 'tool_call' || u?.sessionUpdate === 'tool_call_update';
const clock = () => new Date().toISOString().slice(11, 19);

export function traceAntigravityEvent(method: string, params: unknown): string | null {
  try {
    const p = frameOf(params);
    const at = clock();
    if (method === MESSAGE_DONE) {
      const said = text(p.text).trim();
      return said ? `${at}  ${said}` : null;
    }
    if (method === TURN_DONE) {
      const trouble = turnTrouble(p);
      const reason = text(p.stopReason) || 'error';
      const span = Number.isFinite(p.durationMs) ? `${((p.durationMs as number) / 1000).toFixed(1)}s` : '?s';
      return `\n${at}  == RESULT (${reason}${trouble ? ' ERROR' : ''} · ${span}) ==\n${(trouble ?? text(p.text)).slice(0, 4000)}`;
    }
    if (method !== 'session/update' || !toolUpdate(p.update) || !finished(p.update?.status)) return null;
    const tool = toolWordOf(p.update, STORE_SERVERS);
    if (!tool) return null;
    const failed = p.update?.status === 'failed' ? ' (failed)' : '';
    return `${at}  [${tool.word}]${failed}${tool.detail ? ` ${tool.detail.slice(0, 200)}` : ''}`;
  } catch {
    return null;
  }
}

export function summarizeAntigravityEvent(method: string, params: unknown): string | null {
  try {
    const p = frameOf(params);
    if (method === MESSAGE_DONE) return text(p.text).trim().slice(0, 300) || null;
    if (method === TURN_DONE) return `done: ${(turnTrouble(p) ?? text(p.text)).slice(0, 200)}`;
    if (method !== 'session/update' || !toolUpdate(p.update) || p.update?.status !== 'completed') return null;
    const tool = toolWordOf(p.update, STORE_SERVERS);
    return tool ? `tool: ${tool.word}` : null;
  } catch {
    return null;
  }
}

/**
 * The agy mode for the mode a reply picked, in Claude Code's words, or null for
 * the worker's own (Auto). Nothing maps to skipping its questions.
 */
export function antigravityMode(answerMode: unknown): string | null {
  return ({ plan: 'plan', acceptEdits: 'accept-edits', default: 'default' } as Record<string, string>)[text(answerMode)] ?? null;
}

export interface AcpDiff { type?: unknown; path?: unknown; oldText?: unknown; newText?: unknown }
export interface AcpPermissionParams { sessionId?: unknown; toolCall?: (AcpUpdate & { content?: unknown }) | null; options?: unknown }

const linesOf = (value: unknown) => (typeof value === 'string' && value !== '' ? value.replace(/\n$/, '').split('\n') : []);

/**
 * One permission question as the card the app already draws, or null when it
 * names nothing a person could answer about, which is then refused. The
 * agent's own `toolSummary` is its words about the call, not an argument.
 */
export function antigravityApprovalCard(params: AcpPermissionParams | null | undefined, servers: readonly string[] = []): { tool: string; input: Record<string, string> } | null {
  const call = params?.toolCall;
  if (!call) return null;
  const input = call.rawInput ?? {};
  const name = text(call.name).trim();
  const summary = text(input.toolSummary).trim();

  if (name.startsWith('mcp_')) {
    const tool = antigravityToolName(name, servers);
    if (!tool.startsWith('mcp__')) return null;
    const args: Record<string, string> = {};
    for (const [key, value] of Object.entries(input)) {
      if (key === 'toolSummary') continue;
      args[key] = typeof value === 'string' ? value : JSON.stringify(value);
    }
    return { tool, input: args };
  }

  if (call.kind === 'execute') {
    const command = text(input.CommandLine).trim();
    if (!command) return null;
    const card: Record<string, string> = { command };
    if (text(input.Cwd)) card.cwd = text(input.Cwd);
    const outside = input.BypassSandbox === true;
    if (summary || outside) card.description = [summary, outside ? 'outside the sandbox' : ''].filter(Boolean).join(', ');
    return { tool: 'Bash', input: card };
  }

  if (call.kind === 'edit' || call.kind === 'delete' || call.kind === 'move') {
    // An ACP change block is the one content shape with a path and new text.
    const diffs = (Array.isArray(call.content) ? call.content : []).filter((c): c is AcpDiff => !!c && typeof (c as AcpDiff).newText === 'string' && !!text((c as AcpDiff).path));
    if (!diffs.length) return null;
    const verbs: string[] = [];
    const blocks = diffs.map(d => {
      const verb = call.kind === 'delete' ? 'Delete' : d.oldText == null ? 'Write' : 'Edit';
      if (!verbs.includes(verb)) verbs.push(verb);
      return [`${verb} ${text(d.path)}`, ...linesOf(d.oldText).map(l => `-${l}`), ...linesOf(d.newText).map(l => `+${l}`)].join('\n');
    });
    return { tool: verbs.length === 1 ? verbs[0] : 'Edit', input: { changes: blocks.join('\n\n') } };
  }

  if (call.kind === 'read') {
    const where = text(call.locations?.[0]?.path) || text(input.Path);
    return where ? { tool: 'Read', input: { file_path: where } } : null;
  }

  if (call.kind === 'fetch') {
    const url = text(input.Url) || text(input.url);
    return url ? { tool: 'WebFetch', input: { host: url, ...(summary ? { description: summary } : {}) } } : null;
  }

  return null;
}

/** A file this run changed, kept on the session for the change card. */
export function rememberAntigravityChange(session: AntigravitySession | null | undefined, method: string, params: unknown): void {
  try {
    if (!session || method !== 'session/update') return;
    const u = frameOf(params).update;
    if (!toolUpdate(u) || u?.status !== 'completed') return;
    if (u?.kind !== 'edit' && u?.kind !== 'delete' && u?.kind !== 'move') return;
    for (const location of u.locations ?? []) {
      const file = text(location?.path);
      if (!file) continue;
      session.antigravityChange ??= [];
      if (!session.antigravityChange.includes(file)) session.antigravityChange.push(file);
    }
  } catch { /* a card is never worth a session */ }
}
