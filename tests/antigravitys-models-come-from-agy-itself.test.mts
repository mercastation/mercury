// 2026-11-19: agy keeps no model list on disk, so the picker's list is
// `agy models`, read in the background and kept. Measured on agy 1.5.13: one
// line per model and level, `gemini-4-neon-high<TAB>Gemini 4 Neon (High)`,
// after a "Fetching available models..." line; signed out it exits 1 with
// "Please sign in to view available models", so the same call says whether
// it is signed in. One model's id carries its level with no sibling
// (`gpt-oss-120b-medium`), and is offered as itself.
import { describe, it, expect } from 'vitest';
import { EventEmitter } from 'node:events';
import { antigravityModelRows, readAntigravityModels, antigravityModels, forgetAntigravityModels } from '../main/antigravity-models.mts';

const LISTED = [
  'Fetching available models...',
  'gemini-4-neon-high\tGemini 4 Neon (High)',
  'gemini-4-neon-xhigh\tGemini 4 Neon (XHigh)',
  'gemini-4-neon-medium\tGemini 4 Neon (Medium)',
  'gemini-4-neon-low\tGemini 4 Neon (Low)',
  'gemini-pro-orchestrator-neon\tOrchestrator (Neon)',
  'claude-opus-5-5-low\tClaude Opus 5.5 (Low)',
  'claude-opus-5-5-high\tClaude Opus 5.5 (High)',
  'gpt-oss-120b-medium\tGPT-OSS 120B (Medium)',
  '',
].join('\n');

function fakeAgy(out: string, code: number | null) {
  const child = new EventEmitter() as EventEmitter & { stdout: EventEmitter; stderr: EventEmitter; kill(): void };
  child.stdout = new EventEmitter();
  child.stderr = new EventEmitter();
  child.kill = () => {};
  setImmediate(() => { child.stdout.emit('data', Buffer.from(out)); if (code !== null) child.emit('close', code); });
  return child;
}

describe('Antigravity’s models', () => {
  it('folds each model’s levels into one row, in the order agy lists them', () => {
    expect(antigravityModelRows(LISTED)).toEqual([
      { id: 'gemini-4-neon', label: 'Gemini 4 Neon', levels: ['high', 'xhigh', 'medium', 'low'] },
      { id: 'gemini-pro-orchestrator-neon', label: 'Orchestrator (Neon)', levels: [] },
      { id: 'claude-opus-5-5', label: 'Claude Opus 5.5', levels: ['low', 'high'] },
      { id: 'gpt-oss-120b-medium', label: 'GPT-OSS 120B (Medium)', levels: [] },
    ]);
  });

  it('reads nothing out of a line that is not a model', () => {
    expect(antigravityModelRows('Error: Please sign in to view available models.\n')).toEqual([]);
  });

  it('keeps what agy answered and says it is signed in', async () => {
    forgetAntigravityModels();
    const answer = await readAntigravityModels('/u/agy', { spawn: () => fakeAgy(LISTED, 0) });
    expect(answer.signedIn).toBe(true);
    expect(antigravityModels().map(m => m.id)).toEqual(['gemini-4-neon', 'gemini-pro-orchestrator-neon', 'claude-opus-5-5', 'gpt-oss-120b-medium']);
  });

  it('says it is signed out when agy says so, and keeps the last list', async () => {
    const answer = await readAntigravityModels('/u/agy', { spawn: () => fakeAgy('Error: Please sign in to view available models.\n', 1) });
    expect(answer.signedIn).toBe(false);
    expect(antigravityModels()).toHaveLength(4);
  });

  it('says nothing when agy never answers', async () => {
    const answer = await readAntigravityModels('/u/agy', { spawn: () => fakeAgy('', null), timeoutMs: 20 });
    expect(answer.signedIn).toBeNull();
  });

  it('asks nothing of a Mac without agy', async () => {
    const answer = await readAntigravityModels(null, { spawn: () => { throw Error('must not run'); } });
    expect(answer.signedIn).toBeNull();
  });
});
