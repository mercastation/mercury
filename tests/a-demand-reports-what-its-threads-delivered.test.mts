// A DEMAND REPORTS WHAT ITS THREADS DELIVERED.
//
// What was missing: the inbox says what each thread did, step by step, and
// nothing said what the piece of work as a whole had delivered. On one
// person's store 75 of 97 threads already carried a Context and a Done list,
// so the material was there and nobody had to write anything. The app asks its
// small model to turn a demand's threads into three things: what the demand is,
// what has been delivered so far, and, once every thread is done, the outcome.
//
// Covered here: when a report is due and when it is not, the prompt, every
// part of a reply and the replies that are refused, and the app writing one.
import { describe, it, expect, vi, beforeEach, afterAll } from 'vitest';
import { mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { reportPrompt, readReport, wantsReport, REPORT_EVERY_MS } from '../main/demand-report.mts';
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
  id, product: 'shop', kind: 'directive', status: 'done', title: 'rewrite the refund email', body: '', label: 'Refund email rewrite',
  problem: 'The refund email promised five days and the page said ten.', progress: 'Read both texts\nRewrote the email to say ten days\nShipped it',
  labels: ['founder'], priority: 5, createdAt: NOW - 2 * DAY, updatedAt: NOW - 30 * MIN, claim: null, ...over,
});
const demand = (over: Record<string, unknown> = {}) => ({ id: 'd-1', name: 'Refunds a customer can follow', threads: ['w-1'], createdAt: NOW - 3 * DAY, updatedAt: NOW - 3 * DAY, ...over });

describe('when a demand\'s report is due', () => {
  it('is when a thread of it has said something since the last one', () => {
    expect(wantsReport(demand(), [thread('w-1')], NOW)).toBe(true);
    expect(wantsReport(demand({ wroteAt: NOW - 20 * MIN }), [thread('w-1', { updatedAt: NOW - 10 * MIN })], NOW)).toBe(true);
  });

  it('is not when nothing moved since it was written', () => {
    expect(wantsReport(demand({ wroteAt: NOW - 20 * MIN }), [thread('w-1', { updatedAt: NOW - 30 * MIN })], NOW)).toBe(false);
  });

  it('is not more often than every ten minutes, however busy its threads are', () => {
    const busy = [thread('w-1', { updatedAt: NOW - MIN })];
    expect(wantsReport(demand({ wroteAt: NOW - REPORT_EVERY_MS + MIN }), busy, NOW)).toBe(false);
    expect(wantsReport(demand({ wroteAt: NOW - REPORT_EVERY_MS }), busy, NOW)).toBe(true);
  });

  it('is not while no thread has written a Context, a step or an answer', () => {
    expect(wantsReport(demand(), [thread('w-1', { problem: '', progress: '', result: '' })], NOW)).toBe(false);
    expect(wantsReport(demand({ threads: [] }), [], NOW)).toBe(false);
  });
});

describe('the question', () => {
  const threads = [thread('w-1'), thread('w-2', { status: 'open', label: 'Refund page copy', problem: 'The page still says five days in one place.', progress: 'Found the line' })];

  it('hands over each thread with its state, its Context and its steps', () => {
    const prompt = reportPrompt(demand({ threads: ['w-1', 'w-2'] }), threads);
    expect(prompt).toContain('Refunds a customer can follow');
    expect(prompt).toContain('[done] Refund email rewrite');
    expect(prompt).toContain('[open] Refund page copy');
    expect(prompt).toContain('The refund email promised five days and the page said ten.');
    expect(prompt).toContain('Read both texts; Rewrote the email to say ten days; Shipped it');
  });

  it('asks for the outcome only when every thread is done', () => {
    expect(reportPrompt(demand({ threads: ['w-1', 'w-2'] }), threads)).not.toContain('OUTCOME:');
    expect(reportPrompt(demand(), [thread('w-1')])).toContain('OUTCOME:');
  });

  it('asks for the language the threads are written in', () => {
    expect(reportPrompt(demand(), [thread('w-1')]).replace(/\s+/g, ' ')).toMatch(/in the language the threads are written in/);
  });
});

describe('the answer', () => {
  it('is read part by part', () => {
    expect(readReport('ABOUT: The refund texts agree with each other.\nDELIVERED:\n- Refund email says ten days\n- Refund page corrected\nOUTCOME: A customer reads the same promise everywhere.')).toEqual({
      about: 'The refund texts agree with each other.',
      delivered: ['Refund email says ten days', 'Refund page corrected'],
      outcome: 'A customer reads the same promise everywhere.',
    });
  });

  it('is kept when nothing has been delivered yet', () => {
    expect(readReport('ABOUT: The refund texts agree with each other.\nDELIVERED:\n')).toEqual({ about: 'The refund texts agree with each other.', delivered: [] });
  });

  it('never ends in the middle of a sentence, however long the model ran', () => {
    const long = `OUTCOME: ${'The refund email and the page now promise the same ten days. '.repeat(8)}`;
    const { outcome } = readReport(`ABOUT: Refund texts.\nDELIVERED:\n- One\n${long}`)!;
    expect(outcome!.length).toBeLessThanOrEqual(320);
    expect(outcome!.endsWith('ten days.')).toBe(true);
    const line = `- ${'Refund email rewritten to promise ten days and '.repeat(6)}shipped`;
    expect(readReport(`DELIVERED:\n${line}`)!.delivered[0].length).toBeLessThanOrEqual(140);
  });

  it('is refused when it has no part at all', () => {
    expect(readReport('Sure, here is a summary of the demand.')).toBeNull();
    expect(readReport('')).toBeNull();
  });
});

describe('the app, writing a report', () => {
  const dirs: string[] = [];
  afterAll(() => { for (const d of dirs) { try { rmSync(d, { recursive: true, force: true }); } catch { /* best effort */ } } });
  beforeEach(() => { fake.spawns.length = 0; fake.reply = ''; });

  function build(items: Record<string, unknown>[]) {
    const dir = mkdtempSync(join(tmpdir(), 'report-'));
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

  it('writes what the model said onto the demand, and when', async () => {
    const items = [thread('w-1')];
    const { sup, demands, dir } = build(items);
    const made = demands.make(dir, { name: 'Refunds a customer can follow' }, NOW - 3 * DAY);
    demands.place(dir, 'w-1', made.id, { source: 'agent' }, NOW - 2 * DAY);
    fake.reply = 'ABOUT: The refund texts agree.\nDELIVERED:\n- Refund email says ten days\nOUTCOME: One promise everywhere.\n';
    await sup.reportTheDemands(items as never, NOW);
    expect(demands.get(dir, made.id)).toMatchObject({ about: 'The refund texts agree.', delivered: ['Refund email says ten days'], outcome: 'One promise everywhere.', wroteAt: NOW });
  });

  it('keeps the words a person gave the demand, and adds only what was delivered', async () => {
    const items = [thread('w-1')];
    const { sup, demands, dir } = build(items);
    const made = demands.make(dir, { name: 'Refunds a customer can follow', about: 'My own words.' }, NOW - 3 * DAY);
    demands.place(dir, 'w-1', made.id, { source: 'agent' }, NOW - 2 * DAY);
    fake.reply = 'ABOUT: The model\'s words.\nDELIVERED:\n- Refund email says ten days\n';
    await sup.reportTheDemands(items as never, NOW);
    expect(demands.get(dir, made.id)).toMatchObject({ about: 'My own words.', delivered: ['Refund email says ten days'] });
  });

  it('does not ask again until a thread says something new', async () => {
    const items = [thread('w-1')];
    const { sup, demands, dir } = build(items);
    const made = demands.make(dir, { name: 'Refunds a customer can follow' }, NOW - 3 * DAY);
    demands.place(dir, 'w-1', made.id, { source: 'agent' }, NOW - 2 * DAY);
    fake.reply = 'ABOUT: The refund texts agree.\nDELIVERED:\n- Refund email says ten days\n';
    await sup.reportTheDemands(items as never, NOW);
    await sup.reportTheDemands(items as never, NOW + 30 * MIN);
    expect(fake.spawns).toHaveLength(1);
  });

  it('leaves the demand as it was after a reply it cannot read', async () => {
    const items = [thread('w-1')];
    const { sup, demands, dir } = build(items);
    const made = demands.make(dir, { name: 'Refunds a customer can follow' }, NOW - 3 * DAY);
    demands.place(dir, 'w-1', made.id, { source: 'agent' }, NOW - 2 * DAY);
    fake.reply = 'Happy to help!\n';
    await sup.reportTheDemands(items as never, NOW);
    expect(demands.get(dir, made.id)!.wroteAt).toBeUndefined();
    await sup.reportTheDemands(items as never, NOW + 15_000);
    expect(fake.spawns).toHaveLength(1);
  });
});
