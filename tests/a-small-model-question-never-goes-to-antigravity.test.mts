// 2026-11-21 review. The app's short questions to a small model (naming a
// thread, placing it in a demand, writing a report) try Claude Code, then every
// other agent this Mac offers. On Antigravity that call could not work: it
// passed Claude's `haiku`, which agy refuses ("model haiku is not recognized",
// measured on agy 1.5.13), and `agy -p` takes no home of the app's own, so every
// question would also have been saved in the person's own agy history. agy now
// declares the operation missing and is not asked.
import { describe, it, expect } from 'vitest';
import { Supervisor } from '../main/supervisor.mts';
import { supportsHarnessOperation } from '../main/harnesses.mts';

describe('a small-model question', () => {
  it('is an operation Antigravity declares missing', () => {
    expect(supportsHarnessOperation('antigravity', 'smallModelArgs')).toBe(false);
    expect(supportsHarnessOperation('claude', 'smallModelArgs')).toBe(true);
    expect(supportsHarnessOperation('codex', 'smallModelArgs')).toBe(true);
  });

  it('goes to Claude Code and Codex, never to Antigravity, even when they answer nothing', async () => {
    const asked: string[] = [];
    const sup = Object.assign(Object.create(Supervisor.prototype), {
      config: { claudeBin: '/c', codexBin: '/x' },
      _homeEngine: () => 'claude',
      _projectProfile: () => null,
      engineChoices: () => [{ id: 'claude' }, { id: 'codex' }, { id: 'antigravity' }],
      _smallModelEnvs: () => [{}],
    });
    const answer = await sup._askSmall(async ({ engine }: { engine: string }) => { asked.push(engine); return ''; });
    expect(answer).toBe('');
    expect(asked).toEqual(['claude', 'codex']);
  });
});
