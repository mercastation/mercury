// A NEW THREAD IS PUT IN A DEMAND WITHOUT THE PERSON BEING ASKED.
//
// Why: a list of work kept by hand had already been tried by the person this
// was built for, in another app, and went unopened. So nobody files a thread
// under a demand. The app asks its small model once per thread: which of this
// project's demands does it serve, a new one (and its name), or none. A thread
// filed by another thread goes where that one is, with no model call at all.
//
// Covered here: which threads are asked about and which are not, the prompt,
// every shape of reply and the replies that are refused, and the app acting on
// each of the three answers, on a filed thread, and never over the person.
import { describe, it, expect, vi, beforeEach, afterAll } from 'vitest';
import { mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { placePrompt, readPlace, wantsPlace, PLACE_WINDOW_MS, PLACE_AFTER_MS } from '../main/demand-sorter.mts';
import { DemandStore } from '../main/demands.mts';

const fake = vi.hoisted(() => ({ spawns: [] as { args: string[] }[], reply: '' }));
vi.mock('node:child_process', async (importActual) => {
  const actual = await importActual<typeof import('node:child_process')>();
  const { EventEmitter: Emitter } = await import('node:events');
  return {
    ...actual,
    spawn: (_bin: string, args: string[]) => {
      fake.spawns.push({ args });
      const child = new Emitter() as InstanceType<typeof Emitter> & { stdout: InstanceType<typeof Emitter>; stderr: InstanceType<typeof Emitter>; kill: () => boolean };
      child.stdout = new Emitter();
      child.stderr = new Emitter();
      child.kill = () => true;
      setTimeout(() => { child.stdout.emit('data', Buffer.from(fake.reply)); child.emit('close', 0); }, 0);
      return child;
    },
  };
});

const { Supervisor } = await import('../main/supervisor.mts');

const NOW = 1_795_000_000_000;
const MIN = 60_000;
const DAY = 86_400_000;
const thread = (id: string, over: Record<string, unknown> = {}) => ({
  id, product: 'shop', kind: 'directive', status: 'open', title: 'the refund email reads badly, rewrite it', body: '', label: 'Refund email rewrite',
  labels: ['founder'], priority: 5, createdAt: NOW - 30 * MIN, updatedAt: NOW - 30 * MIN, claim: null, ...over,
});
const flat = (s: string) => s.replace(/\s+/g, ' ');

describe('a thread that is asked about', () => {
  const none = new Set<string>();

  it('is one nobody has placed, once it has a name', () => {
    expect(wantsPlace(thread('w-1'), none, NOW)).toBe(true);
  });

  it('waits for its name, a quarter of an hour at most', () => {
    const unnamed = thread('w-1', { label: '', createdAt: NOW - MIN, updatedAt: NOW - MIN });
    expect(wantsPlace(unnamed, none, NOW)).toBe(false);
    expect(wantsPlace({ ...unnamed, createdAt: NOW - PLACE_AFTER_MS }, none, NOW)).toBe(true);
  });

  it('is not one that already has a place, a demand or none', () => {
    expect(wantsPlace(thread('w-1'), new Set(['w-1']), NOW)).toBe(false);
  });

  it('is not one nobody has touched for two weeks', () => {
    expect(wantsPlace(thread('w-1', { updatedAt: NOW - PLACE_WINDOW_MS + MIN }), none, NOW)).toBe(true);
    expect(wantsPlace(thread('w-1', { updatedAt: NOW - PLACE_WINDOW_MS - MIN }), none, NOW)).toBe(false);
  });

  it('is not a recorded meeting, whose tasks are each asked about on their own', () => {
    expect(wantsPlace(thread('w-1', { body: 'I recorded a meeting: [Sprint planning](transcripts/2026-11-06-1817-sprint-planning.md) (42 min).' }), none, NOW)).toBe(false);
    expect(wantsPlace(thread('w-1', { body: 'I recorded a meeting on my phone: [Standup](transcripts/2026-11-06-0900-standup.md) (9 min). The audio is a.m4a.' }), none, NOW)).toBe(false);
    expect(wantsPlace(thread('w-1', { body: 'Notes from when I recorded a meeting with the supplier.' }), none, NOW)).toBe(true);
  });

  it('is not a chat, a teammate\'s run, a run of a repeating task, a question, a review or a cause', () => {
    for (const over of [{ labels: ['from-chat'] }, { labels: ['teammate:t-1'] }, { labels: ['repeat:r-1'] }, { labels: ['first-run'] }, { kind: 'question' }, { kind: 'review' }, { kind: 'cause' }]) {
      expect(wantsPlace(thread('w-1', over), none, NOW), JSON.stringify(over)).toBe(false);
    }
  });
});

describe('the question', () => {
  const demands = [{ name: 'Refunds a customer can follow', about: 'Emails and the refund page say the same thing.' }, { name: 'Pricing page' }];

  it('lists the project\'s demands by number, with what each is', () => {
    const prompt = placePrompt(thread('w-1', { problem: 'The refund email promises five days; the page says ten.' }), demands);
    expect(prompt).toContain('1. Refunds a customer can follow: Emails and the refund page say the same thing.');
    expect(prompt).toContain('2. Pricing page');
    expect(prompt).toContain('Refund email rewrite');
    expect(prompt).toContain('The refund email promises five days; the page says ten.');
  });

  it('asks for a number, NEW with a name, or NONE', () => {
    const prompt = flat(placePrompt(thread('w-1'), demands));
    expect(prompt).toMatch(/the number of the demand/);
    expect(prompt).toMatch(/NEW: /);
    expect(prompt).toMatch(/NONE/);
    expect(prompt).toMatch(/in the language the thread is written in/);
  });

  it('asks for a demand wide enough for the threads that will follow, in the language the demands already use', () => {
    const prompt = flat(placePrompt(thread('w-1'), demands));
    expect(prompt).toMatch(/even when it is another step or another ticket of it/);
    expect(prompt).toMatch(/the feature or the goal, never the ticket or the step/);
    expect(prompt).toMatch(/in the language the demands below are named in/);
  });

  it('says so when the project has no demand yet', () => {
    expect(placePrompt(thread('w-1'), [])).toContain('(none yet)');
  });
});

describe('the answer', () => {
  it('is a demand by its number', () => {
    expect(readPlace('2', 3)).toEqual({ to: 'demand', index: 1 });
    expect(readPlace(' 1.\n', 3)).toEqual({ to: 'demand', index: 0 });
  });

  it('is a new demand with its name', () => {
    expect(readPlace('NEW: Refunds a customer can follow', 0)).toEqual({ to: 'new', name: 'Refunds a customer can follow' });
    expect(readPlace('new: "Pricing page."', 2)).toEqual({ to: 'new', name: 'Pricing page' });
  });

  it('is none', () => {
    expect(readPlace('NONE', 2)).toEqual({ to: 'none' });
    expect(readPlace('None.', 2)).toEqual({ to: 'none' });
  });

  it('is refused when it is none of those', () => {
    expect(readPlace('4', 3)).toBeNull();
    expect(readPlace('0', 3)).toBeNull();
    expect(readPlace('NEW:', 3)).toBeNull();
    expect(readPlace('I think it belongs with the pricing work because', 3)).toBeNull();
    expect(readPlace('', 3)).toBeNull();
  });
});

describe('the app, placing a thread', () => {
  const dirs: string[] = [];
  afterAll(() => { for (const d of dirs) { try { rmSync(d, { recursive: true, force: true }); } catch { /* best effort */ } } });
  beforeEach(() => { fake.spawns.length = 0; fake.reply = ''; });

  function build(items: Record<string, unknown>[]) {
    const dir = mkdtempSync(join(tmpdir(), 'place-'));
    dirs.push(dir);
    writeFileSync(join(dir, 'project.json'), JSON.stringify({ schemaVersion: 1, id: 'p', name: 'Shop' }));
    const product = { slug: 'shop', name: 'Shop', dir, repoPath: null };
    const demands = new DemandStore();
    const sup = new Supervisor(
      { home: '/nonexistent-home-with-no-discovered-accounts', storeRoot: dir, claudeBin: '/nonexistent/claude', claudeFound: true, maxConcurrentSessions: 3, autonomousProducts: ['shop'] },
      {
        listItems: () => items, listProducts: () => [product], getProduct: () => product, isDue: () => true,
        settleAnswer() {}, recordSessionResult() {}, listRepeats: () => [], demands, productDir: () => dir,
      } as never,
      '/nonexistent-app',
    );
    return { sup, demands, dir };
  }

  it('puts it in the demand the model names', async () => {
    const items = [thread('w-1')];
    const { sup, demands, dir } = build(items);
    demands.make(dir, { name: 'Pricing page' }, NOW - DAY);
    const refunds = demands.make(dir, { name: 'Refunds a customer can follow' }, NOW - DAY + 1);
    fake.reply = '2\n';
    await sup.placeTheThreads(items as never, NOW);
    expect(demands.of(dir, 'w-1')!.id).toBe(refunds.id);
    expect(fake.spawns).toHaveLength(1);
  });

  it('makes the demand the model names when none fits', async () => {
    const items = [thread('w-1')];
    const { sup, demands, dir } = build(items);
    fake.reply = 'NEW: Refunds a customer can follow\n';
    await sup.placeTheThreads(items as never, NOW);
    expect(demands.list(dir).map((d) => d.name)).toEqual(['Refunds a customer can follow']);
    expect(demands.of(dir, 'w-1')!.name).toBe('Refunds a customer can follow');
  });

  it('remembers none, and does not ask again', async () => {
    const items = [thread('w-1')];
    const { sup, demands, dir } = build(items);
    fake.reply = 'NONE\n';
    await sup.placeTheThreads(items as never, NOW);
    expect(demands.isPlaced(dir, 'w-1')).toBe(true);
    await sup.placeTheThreads(items as never, NOW + MIN);
    expect(fake.spawns).toHaveLength(1);
  });

  it('puts a thread another thread filed where that one is, without asking the model', async () => {
    const items = [thread('w-parent'), thread('w-child', { parent: 'w-parent', labels: [], label: 'Refund page copy' })];
    const { sup, demands, dir } = build(items);
    const refunds = demands.make(dir, { name: 'Refunds a customer can follow' }, NOW - DAY);
    demands.place(dir, 'w-parent', refunds.id, { source: 'person' }, NOW - DAY);
    await sup.placeTheThreads(items as never, NOW);
    expect(demands.of(dir, 'w-child')!.id).toBe(refunds.id);
    expect(fake.spawns).toHaveLength(0);
  });

  it('never moves a thread the person placed', async () => {
    const items = [thread('w-1')];
    const { sup, demands, dir } = build(items);
    const pricing = demands.make(dir, { name: 'Pricing page' }, NOW - DAY);
    demands.place(dir, 'w-1', pricing.id, { source: 'person' }, NOW - DAY);
    fake.reply = 'NEW: Something else\n';
    await sup.placeTheThreads(items as never, NOW);
    expect(fake.spawns).toHaveLength(0);
    expect(demands.of(dir, 'w-1')!.id).toBe(pricing.id);
  });

  it('leaves a thread alone after a reply it cannot read, and tries later rather than every tick', async () => {
    const items = [thread('w-1')];
    const { sup, demands, dir } = build(items);
    fake.reply = 'It could be either, honestly.\n';
    await sup.placeTheThreads(items as never, NOW);
    expect(demands.isPlaced(dir, 'w-1')).toBe(false);
    await sup.placeTheThreads(items as never, NOW + 15_000);
    expect(fake.spawns).toHaveLength(1);
  });
});
