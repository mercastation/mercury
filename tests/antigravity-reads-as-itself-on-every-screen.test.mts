// 2026-11-21 review of the Antigravity work, Claude and Codex reading it
// together. Screens that named or chose between "Codex" and "everything else"
// took Antigravity for Claude Code: the home page's tiles ("Claude Code", "C"),
// an approval card from an agy run ("Claude Code" asking), the cloud's reasons
// ("No this agent account"), the inbox filter (no way to pick Antigravity's
// rows), Send now (offered on a run that cannot cut a step, as Codex cannot),
// and the remembered level (Claude Code's, so a level picked for one became the
// other's default).
import { describe, it, expect } from 'vitest';
import fs from 'node:fs';
import { engineLetter, engineName } from '../renderer/src/home/home-model.ts';
import { filterMenu, harnessOf, NO_FILTER } from '../renderer/src/box-filter.ts';
import { whyWords } from '../renderer/src/where-why.ts';
import { cutsARunningStep } from '../renderer/src/engines.ts';
import { LAST_EFFORT_KEY, lastEffortKey, readLastEffort, writeLastEffort } from '../renderer/src/effort.ts';

const memory = () => {
  const map = new Map<string, string>();
  return { map, getItem: (k: string) => map.get(k) ?? null, setItem: (k: string, v: string) => { map.set(k, v); }, removeItem: (k: string) => { map.delete(k); } } as unknown as Storage;
};

describe('Antigravity on the screens', () => {
  it('is named on the home page, with a letter of its own', () => {
    expect(engineName('antigravity')).toBe('Antigravity');
    expect(engineLetter('antigravity')).toBe('A');
    expect([engineName('claude'), engineLetter('claude'), engineName('codex'), engineLetter('codex')]).toEqual(['Claude Code', 'C', 'Codex', 'X']);
  });

  it('is the agent an approval card names', () => {
    const app = fs.readFileSync(new URL('../renderer/src/App.tsx', import.meta.url), 'utf8');
    expect(app).toMatch(/const agent = row \? engineLabel\(harnessOf\(row\)\) : null;/);
  });

  it('is named in the cloud’s reasons', () => {
    expect(whyWords({ at: 'mac', why: 'no-account', detail: 'antigravity' }).body).toBe('No Antigravity account is signed in to the cloud.');
  });

  it('can be picked in the inbox filter once it shares the box with another agent', () => {
    const rows = [
      { product: 'p', priority: 1, labels: [], engine: 'antigravity' },
      { product: 'p', priority: 1, labels: [] },
      { product: 'p', priority: 1, labels: [], model: 'gemini-4-neon' },
    ];
    expect(filterMenu(rows as never, NO_FILTER, []).harnesses.map((h) => [h.value, h.count])).toEqual([['claude', 1], ['antigravity', 2]]);
    expect(filterMenu(rows.slice(0, 1) as never, NO_FILTER, []).harnesses).toEqual([]);
    expect(harnessOf({ labels: [], model: 'gemini-4-neon' } as never)).toBe('antigravity');
  });

  it('is not offered Send now, since agy cannot cut a step', () => {
    expect(cutsARunningStep('claude')).toBe(true);
    expect(cutsARunningStep(undefined)).toBe(true);
    expect(cutsARunningStep('codex')).toBe(false);
    expect(cutsARunningStep('antigravity')).toBe(false);
    const thread = fs.readFileSync(new URL('../renderer/src/components/ItemThread.tsx', import.meta.url), 'utf8');
    expect(thread).toMatch(/onSendNow=\{session && cutsARunningStep\(session\.engine \?\? engine\) \? onSendNow : undefined\}/);
  });

  it('remembers its own level, apart from Claude Code’s', () => {
    const store = memory();
    writeLastEffort('low', store, 'claude');
    writeLastEffort('xhigh', store, 'antigravity');
    expect(lastEffortKey('antigravity')).toBe(`${LAST_EFFORT_KEY}.antigravity`);
    expect(readLastEffort(store, 'antigravity')).toBe('xhigh');
    expect(readLastEffort(store, 'claude')).toBe('low');
  });
});
