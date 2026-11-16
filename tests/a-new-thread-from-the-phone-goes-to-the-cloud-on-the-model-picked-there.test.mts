// A NEW THREAD FROM THE PHONE GOES TO THE CLOUD ON THE MODEL PICKED THERE.
//
// What broke (2026-11-16): a thread started on the phone at 09:22 with the Mac
// asleep stayed queued until the lid opened at 09:43. The phone's new-thread
// card always names a Claude model (`model || DEFAULT_MODEL`, that morning
// "fable"), and a project's new-thread kit said `model: null`, the stand-in
// row's, though its command line ran `--model opus`. So the server left every
// such thread to its computer ("it asks for a model its kit does not run"),
// and the Mac, which took this one in a dark wake (pmset: 09:23:36, 60 s), did
// not hand it over either, without a word. No thread started on the phone went
// to the cloud.
//
// Now the kit says what its command line runs, the line that tells the run so,
// and the models the Mac offers. A pick from the phone is put on the command
// line (`newThreadOn`); only a model or a level the Mac does not offer is left
// to the computer.
import { describe, it, expect } from 'vitest';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { newThreadOn, runIsOnLine, NEW_THREAD } from '../shared/cloud-kit.mts';
import { Supervisor } from '../main/supervisor.mts';
import { kitCopyPath } from '../main/cloud/kit.mts';

const NOTE = 'This run is on Claude Code, Opus 5.5.';
const SYSTEM = `## Connected tools\nNone.\n\n${NOTE}\nCheck a rule that names a model or an effort level against it.`;
const MODELS = { opus: 'Opus 5.5', fable: 'Fable 5.2', sonnet: 'Sonnet 5.5', 'claude-opus-5-5': 'Opus 5.5' };

const kit = (over: Record<string, unknown> = {}, claude: Record<string, unknown> | null = {}) => ({
  v: 1, engine: 'claude' as const, madeAt: 1, cwd: '/r', item: NEW_THREAD,
  argv: ['claude', '-p', '--output-format', 'stream-json', '--model', 'opus', '--allowedTools', 'mcp__mercury', '--append-system-prompt', SYSTEM, '--input-format', 'stream-json'],
  frame: { before: '', after: '' }, login: { engine: 'claude' as const, key: 'k' }, marker: {},
  newThread: {
    brief: 'B', mainSha: 'c'.repeat(40), model: null, effort: null,
    ...(claude ? { claude: { model: 'opus', effort: null, note: NOTE, models: MODELS, ...claude } } : {}),
  },
  ...over,
});
const flag = (argv: string[], name: string) => argv.flatMap((a, i) => (a === name ? [argv[i + 1]] : []));
const system = (argv: string[]) => argv[argv.indexOf('--append-system-prompt') + 1];
const ran = (r: ReturnType<typeof newThreadOn>) => {
  if ('why' in r) throw new Error(`left to the computer: ${r.why}`);
  return r.kit;
};

describe('a new thread\'s kit, on the model and level picked on the phone', () => {
  it('runs a model the Mac offers in place of the one it was packed with, and tells the run so', () => {
    const k = ran(newThreadOn(kit(), { model: 'fable' }));
    expect(flag(k.argv, '--model')).toEqual(['fable']);
    expect(system(k.argv)).toContain('This run is on Claude Code, Fable 5.2.\nCheck a rule');
    expect(system(k.argv)).not.toContain('Opus');
    expect(k.newThread).toMatchObject({ model: 'fable', effort: null });
  });

  it('runs the level picked beside it', () => {
    const k = ran(newThreadOn(kit(), { model: 'fable', effort: 'xhigh' }));
    const at = k.argv.indexOf('--model');
    expect(k.argv.slice(at, at + 4)).toEqual(['--model', 'fable', '--effort', 'xhigh']);
    expect(system(k.argv)).toContain('This run is on Claude Code, Fable 5.2, at xhigh effort.');
  });

  it('puts a new level in place of the one on the line, never beside it', () => {
    const packed = kit({ argv: [...kit().argv.slice(0, 6), '--effort', 'high', ...kit().argv.slice(6)] }, { effort: 'high', note: 'This run is on Claude Code, Opus 5.5, at high effort.' });
    (packed.argv as string[])[packed.argv.indexOf('--append-system-prompt') + 1] = SYSTEM.replace(NOTE, 'This run is on Claude Code, Opus 5.5, at high effort.');
    const k = ran(newThreadOn(packed, { effort: 'low' }));
    expect(flag(k.argv, '--effort')).toEqual(['low']);
    expect(flag(k.argv, '--model')).toEqual(['opus']);
    expect(system(k.argv)).toContain('This run is on Claude Code, Opus 5.5, at low effort.');
  });

  it('is the kit as it was for the model and level it runs, or for none picked', () => {
    expect(ran(newThreadOn(kit(), { model: 'opus' })).argv).toEqual(kit().argv);
    expect(ran(newThreadOn(kit(), {})).argv).toEqual(kit().argv);
    expect(ran(newThreadOn(kit(), { model: '', effort: null })).argv).toEqual(kit().argv);
  });

  it('leaves to the computer a model or a level the Mac does not offer', () => {
    expect(newThreadOn(kit(), { model: 'claude-haiku-4-5' })).toHaveProperty('why');
    expect(newThreadOn(kit(), { model: 'fable', effort: 'ultra' })).toHaveProperty('why');
    expect(newThreadOn(kit(), { model: 42 })).toHaveProperty('why');
  });

  it('leaves to the computer a pick a kit packed before this change cannot take, and runs its own', () => {
    expect(newThreadOn(kit({}, null), { model: 'fable' })).toHaveProperty('why');
    expect(newThreadOn(kit({}, null), { effort: 'high' })).toHaveProperty('why');
    expect(ran(newThreadOn(kit({}, null), {})).argv).toEqual(kit().argv);
  });

  it('leaves a Codex kit to run only what it was packed with', () => {
    const codex = kit({ engine: 'codex' }, null);
    expect(newThreadOn(codex, { model: 'gpt-6-astra' })).toHaveProperty('why');
    expect(ran(newThreadOn(codex, {})).engine).toBe('codex');
  });

  it('says what a run is on in the words a run on the Mac is told', () => {
    expect(runIsOnLine('claude', 'Fable 5.2', null)).toBe('This run is on Claude Code, Fable 5.2.');
    expect(runIsOnLine('claude', 'Fable 5.2', 'max')).toBe('This run is on Claude Code, Fable 5.2, at max effort.');
    expect(runIsOnLine('claude', null, null)).toBeNull();
  });
});

/* -------------------------------- the Mac -------------------------------- */

const MIN = 60_000;
const DEV = 'a1b2c3d4e5f6';

function asleepWith(items: Record<string, unknown>[]) {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'new-on-model-'));
  const store = {
    listItems: () => items,
    listProducts: () => [{ slug: 'alpha', name: 'Alpha', dir: '/tmp/nowhere' }],
    isDue: (i: { runAt?: number }, now = Date.now()) => !i?.runAt || i.runAt <= now,
  };
  const sup = new Supervisor({ storeRoot: root, maxConcurrentSessions: 3 } as never, store as never, root, root, root);
  sup.spawnWorker = (() => null) as never;
  sup.idleMs = () => 30 * MIN;
  sup._phoneCarries = { device: DEV, slugs: new Set(['alpha']) } as never;
  sup.leavesKitsFor = (slug: string) => slug === 'alpha';
  fs.mkdirSync(path.join(root, 'cloud-kits'), { recursive: true });
  fs.writeFileSync(kitCopyPath(root, 'alpha', NEW_THREAD), JSON.stringify(kit({ project: 'alpha' })));
  sup._lastTickAt = Date.now() - 16 * MIN;
  return sup;
}
const row = (extra: Record<string, unknown>) => ({ id: 'w-1', product: 'alpha', status: 'open', kind: 'directive', labels: ['founder'], title: 'KOMS-1474', body: 'Think it through.', createdAt: 1000, updatedAt: 1000, ...extra });

describe('a sleeping Mac and a thread the phone started on a model', () => {
  it('hands it to the cloud with the model and level the phone picked', async () => {
    const sup = asleepWith([row({ model: 'fable', effort: 'high' })]);
    const list = await sup.carryWhileAsleep();
    expect(list).toEqual([expect.objectContaining({ item: 'w-1', kind: 'run-now', payload: expect.objectContaining({ model: 'fable', effort: 'high' }) })]);
  });

  it('hands over one on the model its kit runs', async () => {
    const sup = asleepWith([row({ model: 'opus' })]);
    expect(await sup.carryWhileAsleep()).toHaveLength(1);
  });

  it('keeps one on a model it does not offer', async () => {
    const sup = asleepWith([row({ model: 'claude-haiku-4-5' })]);
    expect(await sup.carryWhileAsleep()).toEqual([]);
  });
});
