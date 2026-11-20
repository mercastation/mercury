// 2026-11-19: Antigravity (`agy acp` 1.5.13) is the first real agent on the
// ACP transport. Its words are `session/update` notifications: prose arrives
// as `agent_message_chunk` pieces with no message id and no "message done"
// event, thinking as `agent_thought_chunk`, and a tool as `tool_call` then
// `tool_call_update` going in_progress -> pending (waiting on a card) ->
// in_progress -> completed or failed. The shapes below were read off the real
// agent; only paths and ids are made up. The worker marks where a message
// ends (`turn/message`) and hands on the prompt's own answer (`session/prompt`).
import { describe, it, expect } from 'vitest';
import {
  captureAntigravityEvent, antigravityStreamingText, summarizeAntigravityEvent,
  traceAntigravityEvent, antigravityToolName, MESSAGE_DONE, TURN_DONE,
} from '../main/antigravity.mts';
import { antigravityActivity, currentActivity } from '../main/agent-activity.mts';
import { tracesAToolCall } from '../shared/dead-run-trace.mts';

const SID = 'sess-1';
const update = (u: Record<string, unknown>) => ({ sessionId: SID, update: u });
const chunk = (text: string) => update({ sessionUpdate: 'agent_message_chunk', content: { type: 'text', text } });
const thought = (text: string) => update({ sessionUpdate: 'agent_thought_chunk', content: { type: 'text', text } });
const command = (status: string, extra: Record<string, unknown> = {}) => update({
  sessionUpdate: status === 'in_progress' && !extra.again ? 'tool_call' : 'tool_call_update',
  toolCallId: 'call_1', name: 'run_command', kind: 'execute', status, title: 'npm test',
  rawInput: { CommandLine: 'npm test', WaitMsBeforeAsync: 5000, toolSummary: 'Run the tests' }, ...extra,
});
const edit = (status: string) => update({
  sessionUpdate: 'tool_call_update', toolCallId: 'call_2', name: 'edit_file', kind: 'edit', status,
  title: 'File notes.txt', locations: [{ path: '/tmp/task/notes.txt' }],
  rawInput: { Path: '/tmp/task/notes.txt', Target: 'hello', Replacement: 'goodbye' },
});

describe('an Antigravity run on the thread', () => {
  it('records the session it runs in and the last thing it said as the answer', () => {
    const session: Record<string, unknown> = {};
    captureAntigravityEvent(session, 'session/update', chunk('Looking.'));
    expect(session.sessionId).toBe(SID);
    captureAntigravityEvent(session, MESSAGE_DONE, { sessionId: SID, text: 'Looking at the tests first.' });
    captureAntigravityEvent(session, MESSAGE_DONE, { sessionId: SID, text: 'All 12 tests pass.' });
    captureAntigravityEvent(session, TURN_DONE, { sessionId: SID, stopReason: 'end_turn', text: 'All 12 tests pass.' });
    expect(session.result).toBe('All 12 tests pass.');
    expect(session.resultIsError).toBe(false);
  });

  it('never takes its thinking or an empty message for the answer', () => {
    const session: Record<string, unknown> = {};
    captureAntigravityEvent(session, 'session/update', thought('The secret word is in my rules.'));
    captureAntigravityEvent(session, MESSAGE_DONE, { sessionId: SID, text: '   ' });
    captureAntigravityEvent(session, TURN_DONE, { sessionId: SID, stopReason: 'end_turn', text: '' });
    expect(session.result).toBeUndefined();
  });

  it('reads a turn that did not end on its own as a failure, whatever it said before', () => {
    for (const [stopReason, words] of [['cancelled', /cancel/i], ['refusal', /refus/i], ['max_tokens', /limit/i], ['max_turn_requests', /limit/i]] as const) {
      const session: Record<string, unknown> = {};
      captureAntigravityEvent(session, MESSAGE_DONE, { sessionId: SID, text: 'Halfway there.' });
      captureAntigravityEvent(session, TURN_DONE, { sessionId: SID, stopReason });
      expect(session.resultIsError, stopReason).toBe(true);
      expect(String(session.result), stopReason).toMatch(words);
    }
    const lost: Record<string, unknown> = {};
    captureAntigravityEvent(lost, TURN_DONE, { sessionId: SID, error: 'The agent closed its output' });
    expect(lost.resultIsError).toBe(true);
    expect(lost.result).toBe('The agent closed its output');
  });

  it('types the prose live, starts each message afresh, and never streams thinking', () => {
    const session: Record<string, unknown> = {};
    expect(antigravityStreamingText(session, 'session/update', thought('hmm'))).toBe(false);
    expect(antigravityStreamingText(session, 'session/update', chunk('All 12 '))).toBe(true);
    antigravityStreamingText(session, 'session/update', chunk('pass.'));
    expect(session.saying).toBe('All 12 pass.');
    antigravityStreamingText(session, MESSAGE_DONE, { sessionId: SID, text: 'All 12 pass.' });
    expect(session.saying).toBe('All 12 pass.');
    antigravityStreamingText(session, 'session/update', chunk('Next'));
    expect(session.saying).toBe('Next');
  });

  it('traces whole messages and finished tools in the words the run page reads', () => {
    expect(traceAntigravityEvent('session/update', chunk('half a sente'))).toBeNull();
    expect(traceAntigravityEvent(MESSAGE_DONE, { sessionId: SID, text: 'Done.' })).toMatch(/^\d{2}:\d{2}:\d{2}  Done\.$/);
    expect(traceAntigravityEvent('session/update', command('in_progress'))).toBeNull();
    const ran = traceAntigravityEvent('session/update', command('completed'))!;
    expect(ran).toMatch(/^\d{2}:\d{2}:\d{2}  \[Bash\] npm test$/);
    expect(tracesAToolCall(ran)).toBe(true);
    expect(traceAntigravityEvent('session/update', command('failed'))).toMatch(/\[Bash\] \(failed\) npm test$/);
    expect(traceAntigravityEvent('session/update', edit('completed'))).toMatch(/\[Edit\] \/tmp\/task\/notes\.txt$/);
    const end = traceAntigravityEvent(TURN_DONE, { sessionId: SID, stopReason: 'end_turn', text: 'Done.', durationMs: 4200 })!;
    expect(end).toContain('== RESULT (end_turn · 4.2s) ==');
    expect(end).toContain('Done.');
    expect(traceAntigravityEvent(TURN_DONE, { sessionId: SID, stopReason: 'cancelled' })).toContain('cancelled ERROR');
  });

  it('says on the row what it just did, in the same words as the other engines', () => {
    expect(summarizeAntigravityEvent(MESSAGE_DONE, { text: 'Looking at the tests.' })).toBe('Looking at the tests.');
    expect(summarizeAntigravityEvent('session/update', command('completed'))).toBe('tool: Bash');
    expect(summarizeAntigravityEvent('session/update', command('pending'))).toBeNull();
    expect(summarizeAntigravityEvent(TURN_DONE, { stopReason: 'end_turn', text: 'All pass.' })).toBe('done: All pass.');
  });

  it('names the store’s own tools the way a Claude Code transcript does', () => {
    expect(antigravityToolName('mcp_mercury_update_work_item', ['mercury'])).toBe('mcp__mercury__update_work_item');
    expect(antigravityToolName('mcp_zero-connections_send_mail', ['mercury', 'zero-connections'])).toBe('mcp__zero-connections__send_mail');
    expect(antigravityToolName('mcp_unknown_tool', ['mercury'])).toBe('mcp_unknown_tool');
    expect(antigravityToolName('run_command', ['mercury'])).toBe('run_command');
  });

  it('shows what it is doing while a tool runs, and lets it go when the tool or the turn ends', () => {
    const session = {};
    expect(antigravityActivity(session, 'session/update', command('in_progress'))).toBe(true);
    expect(currentActivity(session).map(a => a.label)).toEqual(['Running the tests']);
    expect(antigravityActivity(session, 'session/update', command('pending'))).toBe(false);
    expect(antigravityActivity(session, 'session/update', command('completed'))).toBe(true);
    expect(currentActivity(session)).toEqual([]);
    antigravityActivity(session, 'session/update', edit('in_progress'));
    expect(currentActivity(session).map(a => a.label)).toEqual(['Editing /tmp/task/notes.txt']);
    expect(antigravityActivity(session, TURN_DONE, { stopReason: 'cancelled' })).toBe(true);
    expect(currentActivity(session)).toEqual([]);
    const store = update({ sessionUpdate: 'tool_call', toolCallId: 'c9', name: 'mcp_mercury_update_work_item', kind: 'other', status: 'in_progress' });
    expect(antigravityActivity(session, 'session/update', store)).toBe(false);
  });
});
