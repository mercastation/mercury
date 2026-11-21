// 2026-11-21, the owner: "acabei de abrir uma thread com o agy: Claude Code
// could not use this task's model on that account's plan". The row was filed
// with model `gemini-4-neon` and no engine. The card remembers each agent's
// last model under its own key, and Antigravity had none: it wrote into
// Claude Code's, so the next card opened on Claude Code (the default agent)
// with a Gemini model, and filed that pair. Three repairs: Antigravity's key of
// its own, Claude Code's key no longer read when it holds another agent's
// model, and a row naming no engine whose model only Antigravity runs is
// Antigravity's, the rule Codex's slugs already had (which also rescues the
// row that was filed).
import { describe, it, expect } from 'vitest';
import { LAST_MODEL_KEY, lastModelKey, readLastModel, writeLastModel } from '../renderer/src/models.ts';
import { harnessOf, recentModels } from '../renderer/src/threads/composer-rules.ts';
import { ENGINE_CHOICE_ENABLED, engineFor, isAntigravityModel, isCodexModel } from '../shared/engines.mts';

const memory = () => {
  const map = new Map<string, string>();
  return { map, getItem: (k: string) => map.get(k) ?? null, setItem: (k: string, v: string) => { map.set(k, v); }, removeItem: (k: string) => { map.delete(k); } } as unknown as Storage & { map: Map<string, string> };
};

describe('the model each agent last ran', () => {
  it('is kept apart for Antigravity, so picking Gemini leaves Claude Code’s alone', () => {
    const store = memory();
    writeLastModel('sonnet', store, 'claude');
    writeLastModel('gemini-4-neon', store, 'antigravity');
    expect(lastModelKey('antigravity')).toBe(`${LAST_MODEL_KEY}.antigravity`);
    expect(readLastModel(store, 'antigravity')).toBe('gemini-4-neon');
    expect(readLastModel(store, 'claude')).toBe('sonnet');
  });

  it('is not read for Claude Code when its key holds another agent’s model, as it did on her Mac', () => {
    const store = memory();
    store.setItem(LAST_MODEL_KEY, 'gemini-4-neon');
    expect(readLastModel(store, 'claude')).toBeNull();
    store.setItem(LAST_MODEL_KEY, 'gpt-5.6-sol');
    expect(readLastModel(store, 'claude')).toBeNull();
    store.setItem(LAST_MODEL_KEY, 'sonnet');
    expect(readLastModel(store, 'claude')).toBe('sonnet');
  });
});

describe('a model only Antigravity runs', () => {
  it('is Gemini’s and GPT-OSS’s, and GPT-OSS is not Codex’s', () => {
    for (const m of ['gemini-4-neon', 'gemini-pro-orchestrator-neon', 'gpt-oss-120b-medium']) expect(isAntigravityModel(m)).toBe(true);
    for (const m of ['claude-opus-5-5', 'sonnet', 'gpt-5.6-sol', 'o3']) expect(isAntigravityModel(m)).toBe(false);
    expect(isCodexModel('gpt-oss-120b-medium')).toBe(false);
    expect(isCodexModel('gpt-5.6-sol')).toBe(true);
  });

  it('makes a row that names no engine Antigravity’s, where this Mac runs it', () => {
    const row = { model: 'gemini-4-neon' };
    expect(engineFor(row, { found: { claude: true, antigravity: true }, enabled: ENGINE_CHOICE_ENABLED })).toBe('antigravity');
    expect(engineFor(row, { found: { claude: true, antigravity: false }, enabled: ENGINE_CHOICE_ENABLED })).toBe('claude');
    expect(engineFor({ model: 'sonnet' }, { found: { claude: true, antigravity: true }, enabled: ENGINE_CHOICE_ENABLED })).toBe('claude');
    expect(engineFor({ model: 'claude-opus-5-5', engine: 'antigravity' }, { found: { claude: true, antigravity: true }, enabled: ENGINE_CHOICE_ENABLED })).toBe('antigravity');
  });

  it('comes back in the card’s recent models as Antigravity’s, not Claude Code’s', () => {
    expect(harnessOf('gemini-4-neon')).toBe('antigravity');
    const recent = recentModels([{ createdAt: 2, model: 'gemini-4-neon' }], { codexOffered: false, antigravityOffered: true });
    expect(recent[0]).toEqual({ engine: 'antigravity', model: 'gemini-4-neon' });
    expect(recent.some((p) => p.engine === 'claude' && p.model === 'gemini-4-neon')).toBe(false);
  });
});
