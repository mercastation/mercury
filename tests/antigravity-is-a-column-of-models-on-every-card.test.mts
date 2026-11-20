// 2026-11-19: Antigravity joined the model menus beside Claude Code and Codex.
// On a card the model is the agent, and two things would have sent an
// Antigravity pick elsewhere: the rules kept only `claude` and `codex` (a
// third agent fell back to Claude Code), and agy offers Claude models under
// Claude Code's own full ids (`claude-opus-5-5`), so a row's model alone
// cannot say which agent it was picked for.
import { describe, it, expect, beforeEach } from 'vitest';
import { setAntigravityModels, antigravityModelRows, engineModelChoices, engineModelLabel, defaultModelFor, CODEX_OWN } from '../renderer/src/models.ts';
import { effortChoicesFor, defaultEffortFor } from '../renderer/src/effort.ts';
import { allModels, recentModels, agentChoices, moreCount } from '../renderer/src/threads/composer-rules.ts';
import { startingEngine } from '../renderer/src/engines.ts';
import { harnessOf } from '../renderer/src/box-filter.ts';

const AGY = [
  { id: 'gemini-4-neon', label: 'Gemini 4 Neon', levels: ['high', 'xhigh', 'medium', 'low'] },
  { id: 'gemini-pro-orchestrator-neon', label: 'Orchestrator (Neon)', levels: [] },
  { id: 'claude-opus-5-5', label: 'Claude Opus 5.5', levels: ['low', 'medium', 'high'] },
];
const THREE = [{ id: 'claude', label: 'Claude Code' }, { id: 'codex', label: 'Codex' }, { id: 'antigravity', label: 'Antigravity' }];

beforeEach(() => setAntigravityModels(AGY));

describe('Antigravity on a card', () => {
  it('keeps only well-formed rows of what agy listed', () => {
    setAntigravityModels([...AGY, { id: '', label: 'x' }, null, { id: 'y' }]);
    expect(antigravityModelRows().map(m => m.id)).toEqual(['gemini-4-neon', 'gemini-pro-orchestrator-neon', 'claude-opus-5-5']);
    setAntigravityModels(undefined);
    expect(antigravityModelRows()).toEqual([]);
  });

  it('sends nothing until a model is picked, and names that as agy’s own choice', () => {
    expect(defaultModelFor('antigravity')).toBe(CODEX_OWN);
    expect(engineModelLabel('antigravity', null)).toBe('Antigravity’s own');
    expect(engineModelLabel('antigravity', 'gemini-4-neon')).toBe('Gemini 4 Neon');
    expect(engineModelChoices('antigravity', 'gemini-9-new').at(-1)).toEqual({ id: 'gemini-9-new', label: 'gemini-9-new' });
  });

  it('offers each model’s own levels, none for a model without, and lights none of its own', () => {
    expect(effortChoicesFor('antigravity', 'gemini-4-neon').map(e => e.id)).toEqual(['high', 'xhigh', 'medium', 'low']);
    expect(effortChoicesFor('antigravity', 'gemini-pro-orchestrator-neon')).toEqual([]);
    expect(effortChoicesFor('antigravity', null)).toEqual([]);
    expect(defaultEffortFor('antigravity', 'gemini-4-neon')).toBeNull();
  });

  it('draws a third column of agy’s models where Antigravity is offered, and not elsewhere', () => {
    const every = allModels({ codexModels: [], codexOffered: true, antigravityOffered: true });
    expect(every.antigravity).toEqual(AGY.map(m => ({ engine: 'antigravity', model: m.id })));
    expect(allModels({ codexModels: [], codexOffered: true }).antigravity).toEqual([]);
    expect(moreCount([], every)).toBe(every.claude.length + AGY.length);
  });

  it('remembers a pick by the agent it was made for, even on a Claude model’s name', () => {
    const items = [{ createdAt: 2, model: 'claude-opus-5-5', engine: 'antigravity' }, { createdAt: 1, model: 'gemini-4-neon', engine: 'antigravity' }];
    expect(recentModels(items, { codexOffered: false, antigravityOffered: true }).slice(0, 2)).toEqual([
      { engine: 'antigravity', model: 'claude-opus-5-5' },
      { engine: 'antigravity', model: 'gemini-4-neon' },
    ]);
    expect(recentModels(items, { codexOffered: false, antigravityOffered: false }).some(p => p.engine === 'antigravity')).toBe(false);
  });

  it('tags rows with all three agents, opens on Antigravity when it was the pick, and counts its threads as its own', () => {
    expect(agentChoices(THREE).map(e => e.id)).toEqual(['claude', 'codex', 'antigravity']);
    expect(startingEngine('antigravity', THREE as never)).toBe('antigravity');
    expect(startingEngine('antigravity', THREE.slice(0, 2) as never)).toBe('claude');
    expect(harnessOf({ engine: 'antigravity', labels: [] })).toBe('antigravity');
  });
});
