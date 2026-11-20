// 2026-11-19: Antigravity asks over ACP's `session/request_permission`,
// offering exactly two options, `allow` (allow_once) and `deny` (reject_once).
// Measured on agy 1.5.13 with the person's own account: Deny was enforced (the
// command reported failed and the file was never made), Allow ran it, and a
// call to an MCP tool asks too, so the store's own tools would have needed a
// card on every claim and checkpoint. Its questions go into the same spool as
// Codex's and are drawn in the same three shapes the card already reads.
import { describe, it, expect, afterAll } from 'vitest';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { createCodexApprovals, ACP_PERMISSION } from '../main/codex-approvals.mts';
import { antigravityApprovalCard } from '../main/antigravity.mts';
import { listPending } from '../main/approvals.mts';

const roots: string[] = [];
afterAll(() => { for (const r of roots) fs.rmSync(r, { recursive: true, force: true }); });
const storeRoot = () => { const r = fs.mkdtempSync(path.join(os.tmpdir(), 'agy-cards-')); roots.push(r); return r; };

const ask = (toolCall: Record<string, unknown>) => ({ sessionId: 's-1', toolCall: { sessionUpdate: 'tool_call', toolCallId: 'call_1', status: 'pending', ...toolCall }, options: [{ optionId: 'allow', name: 'Allow', kind: 'allow_once' }, { optionId: 'deny', name: 'Deny', kind: 'reject_once' }] });
const command = ask({ name: 'run_command', kind: 'execute', title: 'git push origin HEAD', rawInput: { CommandLine: 'git push origin HEAD', Cwd: '/tmp/task', BypassSandbox: true, toolSummary: 'Push the branch' } });
const editing = ask({ name: 'edit_file', kind: 'edit', title: 'File notes.txt', locations: [{ path: '/tmp/task/notes.txt' }], rawInput: { Path: '/tmp/task/notes.txt', Target: 'hello', Replacement: 'goodbye' }, content: [{ type: 'diff', path: '/tmp/task/notes.txt', oldText: 'hello', newText: 'goodbye' }] });
const mcp = (name: string) => ask({ name, kind: 'other', title: `Run ${name}`, rawInput: { to: 'a@example.invalid', toolSummary: 'Send it' } });

describe('an Antigravity question as a card', () => {
  it('draws a command with where it runs, and says when it would leave the sandbox', () => {
    expect(antigravityApprovalCard(command)).toEqual({ tool: 'Bash', input: { command: 'git push origin HEAD', cwd: '/tmp/task', description: 'Push the branch, outside the sandbox' } });
  });

  it('draws an edit as the change it would make', () => {
    expect(antigravityApprovalCard(editing)).toEqual({ tool: 'Edit', input: { changes: 'Edit /tmp/task/notes.txt\n-hello\n+goodbye' } });
    const fresh = ask({ name: 'write_to_file', kind: 'edit', title: 'File new.txt', content: [{ type: 'diff', path: '/tmp/task/new.txt', oldText: null, newText: 'one\ntwo' }] });
    expect(antigravityApprovalCard(fresh)).toEqual({ tool: 'Write', input: { changes: 'Write /tmp/task/new.txt\n+one\n+two' } });
  });

  it('draws a tool of one of its servers as Claude Code’s MCP card, without the agent’s own summary', () => {
    expect(antigravityApprovalCard(mcp('mcp_zero-connections_send_mail'), ['mercury', 'zero-connections'])).toEqual({ tool: 'mcp__zero-connections__send_mail', input: { to: 'a@example.invalid' } });
  });

  it('cannot describe what it does not recognise', () => {
    expect(antigravityApprovalCard(ask({ name: 'mystery', kind: 'other', title: '' }))).toBeNull();
    expect(antigravityApprovalCard(mcp('mcp_elsewhere_tool'), ['mercury'])).toBeNull();
    expect(antigravityApprovalCard(ask({ name: 'run_command', kind: 'execute', rawInput: {} }))).toBeNull();
  });

  it('lets the store’s own tools and the app’s connections through without a card', async () => {
    const cards = createCodexApprovals({ storeRoot: storeRoot() }).scope({ product: 'p', item: 'w-1', storeServer: 'mercury', connectionServers: ['zero-connections'] });
    await expect(cards.handle(ACP_PERMISSION, mcp('mcp_mercury_update_work_item'))).resolves.toBe('accept');
    await expect(cards.handle(ACP_PERMISSION, mcp('mcp_zero-connections_send_mail'))).resolves.toBe('accept');
  });

  it('puts every other question on a card and waits for the answer', async () => {
    const root = storeRoot();
    const spool = createCodexApprovals({ storeRoot: root, pollMs: 60_000 });
    const cards = spool.scope({ product: 'p', item: 'w-1', storeServer: 'mercury' });
    const answer = cards.handle(ACP_PERMISSION, command);
    const [card] = listPending(root);
    expect(card).toMatchObject({ product: 'p', item: 'w-1', tool: 'Bash', input: { command: 'git push origin HEAD' } });
    expect(spool.settle(card.id, false)).toBe(true);
    await expect(answer).resolves.toBe('decline');
  });

  it('refuses out loud what it cannot draw', async () => {
    const said: string[] = [];
    const cards = createCodexApprovals({ storeRoot: storeRoot() }).scope({ product: 'p', item: 'w-1', storeServer: 'mercury', say: line => said.push(line) });
    await expect(cards.handle(ACP_PERMISSION, ask({ name: 'mystery', kind: 'other' }))).resolves.toBe('decline');
    expect(said.join('\n')).toMatch(/refused/);
  });
});
