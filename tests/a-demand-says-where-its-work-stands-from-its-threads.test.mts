// A DEMAND SAYS WHERE ITS WORK STANDS, AND IT READS THAT OFF ITS THREADS.
//
// The inbox answers "what needs me now", thread by thread. It could not answer
// "what am I trying to deliver, and how far along is each thing": on one
// person's store 30 threads had finished in a week and the list showed only
// the 12 still open. A to-do list kept by hand in another app had been tried
// and was not opened, so nothing here is typed by the person: a demand is to
// do, in progress or delivered because of where its threads are in the inbox,
// and for no other reason.
//
// Covered here: each state and the boundary between them, the words on a
// demand's row for every case, which tab a demand is under, the day pick, the
// week's count, and a thread that names its demand.
import { describe, it, expect } from 'vitest';
import { demandRows, demandTabs, demandByThread, finishedThisWeek, demandSentence, deliveredLines, threadsInOrder, wordForPlace, type Place } from '../renderer/src/demands/demand-rules';
import { dayKey } from '../shared/demands.mts';

const NOW = new Date(2026, 10, 20, 12, 0, 0).getTime();
const DAY = 86_400_000;

type Row = { id: string; product: string; productName: string; title: string; label?: string; status: string; updatedAt: number; progress?: string; parent?: string; labels?: string[] };
const thread = (id: string, over: Partial<Row> = {}): Row => ({ id, product: 'shop', productName: 'Shop', title: `Thread ${id}`, status: 'open', updatedAt: NOW - DAY, ...over });
const demand = (id: string, threads: string[], over: Record<string, unknown> = {}) => ({ id, product: 'shop', productName: 'Shop', name: `Demand ${id}`, threads, createdAt: NOW - 9 * DAY, updatedAt: NOW - DAY, ...over });

/** Where the inbox has each thread: the test says so, as the app's own lists do. */
const places = (map: Record<string, Place>) => (i: { id: string }) => map[i.id] ?? null;
const one = (threads: string[], items: Row[], map: Record<string, Place>, over: Record<string, unknown> = {}) =>
  demandRows([demand('d-1', threads, over)] as never, items as never, places(map), NOW)[0];

describe('the state of a demand', () => {
  it('is to do with no thread at all', () => {
    expect(one([], [], {}).state).toBe('todo');
  });

  it('is to do while every thread still waits in Later', () => {
    const row = one(['w-a', 'w-b'], [thread('w-a', { parent: 'w-x' }), thread('w-b', { parent: 'w-x' })], { 'w-a': 'later', 'w-b': 'later' });
    expect(row.state).toBe('todo');
    expect(row.word).toBe('2 to start');
  });

  it('is in progress from the first thread that began', () => {
    expect(one(['w-a', 'w-b'], [thread('w-a'), thread('w-b')], { 'w-a': 'progress', 'w-b': 'later' }).state).toBe('progress');
    expect(one(['w-a', 'w-b'], [thread('w-a', { status: 'done' }), thread('w-b')], { 'w-a': 'done', 'w-b': 'later' }).state).toBe('progress');
    expect(one(['w-a'], [thread('w-a')], { 'w-a': 'needs' }).state).toBe('progress');
  });

  it('is delivered once every thread is done, and not one thread before', () => {
    const done = thread('w-a', { status: 'done' });
    expect(one(['w-a', 'w-b'], [done, thread('w-b', { status: 'done' })], { 'w-a': 'done', 'w-b': 'done' }).state).toBe('delivered');
    expect(one(['w-a', 'w-b'], [done, thread('w-b')], { 'w-a': 'done', 'w-b': 'needs' }).state).toBe('progress');
  });

  it('counts only threads that still exist', () => {
    const row = one(['w-a', 'w-gone'], [thread('w-a', { status: 'done' })], { 'w-a': 'done' });
    expect([row.done, row.total]).toEqual([1, 1]);
    expect(row.state).toBe('delivered');
  });
});

describe('the words on a demand\'s row', () => {
  it('lead with what waits on the person', () => {
    const row = one(['w-a', 'w-b', 'w-c'], [thread('w-a', { label: 'Refund emails' }), thread('w-b'), thread('w-c', { status: 'done' })], { 'w-a': 'needs', 'w-b': 'progress', 'w-c': 'done' });
    expect(row.word).toBe('Needs you');
    expect(row.what).toBe('Refund emails');
    expect(row.waitsOnYou).toBe(true);
    expect([row.done, row.open, row.later]).toEqual([1, 2, 0]);
  });

  it('count them when more than one waits', () => {
    const row = one(['w-a', 'w-b'], [thread('w-a', { label: 'Refund emails' }), thread('w-b', { label: 'Tax lines' })], { 'w-a': 'needs', 'w-b': 'needs' });
    expect(row.word).toBe('2 need you');
    expect(row.what).toBe('Refund emails, and 1 more');
  });

  it('say an agent is working, with the newest step it wrote', () => {
    const row = one(['w-a'], [thread('w-a', { label: 'Refund emails', progress: 'Read the templates\nWrote the new copy' })], { 'w-a': 'progress' });
    expect(row.word).toBe('Working');
    expect(row.what).toBe('Wrote the new copy');
    expect(row.waitsOnYou).toBe(false);
  });

  it('name the working thread when it has written no step yet', () => {
    expect(one(['w-a'], [thread('w-a', { label: 'Refund emails' })], { 'w-a': 'progress' }).what).toBe('Refund emails');
  });

  it('say what is left to start when nothing runs and nothing waits', () => {
    const row = one(['w-a', 'w-b'], [thread('w-a', { status: 'done' }), thread('w-b', { label: 'Tax lines', parent: 'w-a' })], { 'w-a': 'done', 'w-b': 'later' });
    expect(row.word).toBe('1 to start');
    expect(row.what).toBe('Tax lines');
    expect(row.waitsOnYou).toBe(true);
  });

  it('say Later for a thread put off, which is not waiting to be started', () => {
    const row = one(['w-a', 'w-b'], [thread('w-a', { status: 'done' }), thread('w-b', { label: 'Tax lines' })], { 'w-a': 'done', 'w-b': 'later' });
    expect(row.word).toBe('Later');
    expect(row.waitsOnYou).toBe(false);
  });

  it('say when it was delivered and by how many threads', () => {
    const row = one(['w-a', 'w-b'], [thread('w-a', { status: 'done', updatedAt: NOW - 3 * DAY }), thread('w-b', { status: 'done', updatedAt: NOW - DAY })], { 'w-a': 'done', 'w-b': 'done' });
    expect(row.word).toBe('Delivered');
    expect(row.deliveredAt).toBe(NOW - DAY);
    expect(row.what).toBe('2 threads');
    expect(one(['w-a'], [thread('w-a', { status: 'done' })], { 'w-a': 'done' }).what).toBe('1 thread');
  });

  it('say To do, and nothing made up, for a demand with no thread', () => {
    const row = one([], [], {});
    expect([row.word, row.what]).toEqual(['To do', '']);
  });
});

describe('the tab a demand is under', () => {
  const items = [thread('w-a'), thread('w-b', { status: 'done' }), thread('w-c', { parent: 'w-x' })];
  const map: Record<string, Place> = { 'w-a': 'needs', 'w-b': 'done', 'w-c': 'later' };
  const rows = demandRows([
    demand('d-go', ['w-a'], { today: dayKey(NOW) }),
    demand('d-done', ['w-b']),
    demand('d-wait', ['w-c']),
    demand('d-old-pick', ['w-a'], { today: dayKey(NOW - DAY) }),
  ] as never, items as never, places(map), NOW);
  const tabs = demandTabs(rows);
  const ids = (list: { demand: { id: string } }[]) => list.map((r) => r.demand.id);

  it('is Today only on the day it was picked for', () => {
    expect(ids(tabs.today)).toEqual(['d-go']);
  });

  it('is In progress, To do or Delivered by its state, picked or not', () => {
    expect(ids(tabs.progress).sort()).toEqual(['d-go', 'd-old-pick']);
    expect(ids(tabs.todo)).toEqual(['d-wait']);
    expect(ids(tabs.delivered)).toEqual(['d-done']);
  });
});

describe('the week', () => {
  it('counts threads finished in the last seven days, by project, most first', () => {
    const items = [
      thread('w-1', { status: 'done', updatedAt: NOW - DAY }),
      thread('w-2', { status: 'done', updatedAt: NOW - 6 * DAY }),
      thread('w-3', { status: 'done', updatedAt: NOW - 8 * DAY }),
      thread('w-4', { status: 'open', updatedAt: NOW - DAY }),
      thread('w-5', { status: 'done', updatedAt: NOW - 2 * DAY, product: 'studio', productName: 'Studio' }),
    ];
    expect(finishedThisWeek(items as never, NOW)).toEqual({ total: 3, byProject: [{ name: 'Shop', count: 2 }, { name: 'Studio', count: 1 }] });
  });
});

describe('a demand\'s own page', () => {
  const items = [
    thread('w-done-old', { status: 'done', label: 'Refund copy', updatedAt: NOW - 5 * DAY }),
    thread('w-done-new', { status: 'done', label: 'Refund template', updatedAt: NOW - DAY }),
    thread('w-needs', { label: 'Tax lines' }),
    thread('w-agent', { label: 'Receipt email' }),
    thread('w-start', { label: 'Invoice footer', parent: 'w-needs' }),
    thread('w-later', { label: 'Old receipts' }),
  ];
  const map: Record<string, Place> = { 'w-done-old': 'done', 'w-done-new': 'done', 'w-needs': 'needs', 'w-agent': 'progress', 'w-start': 'later', 'w-later': 'later' };
  const all = ['w-done-old', 'w-needs', 'w-later', 'w-agent', 'w-start', 'w-done-new'];

  it('says in one sentence what waits on the person, then how far along it is', () => {
    expect(demandSentence(one(all, items, map))).toBe('1 thread needs you. 2 of 6 are done.');
    expect(demandSentence(one(['w-needs', 'w-agent'], [...items, thread('w-agent', {})], { ...map }))).toBe('1 thread needs you. 0 of 2 are done.');
    expect(demandSentence(one(['w-agent', 'w-done-new'], items, map))).toBe('An agent is on 1 thread. 1 of 2 is done.');
    expect(demandSentence(one(['w-start', 'w-done-new'], items, map))).toBe('1 thread waits to start. 1 of 2 is done.');
    expect(demandSentence(one(['w-later'], items, map))).toBe('Waiting in Later. 0 of 1 is done.');
  });

  it('says so when it is delivered, and when it has no thread', () => {
    expect(demandSentence(one(['w-done-old', 'w-done-new'], items, map))).toBe('All 2 threads are done.');
    expect(demandSentence(one(['w-done-new'], items, map))).toBe('Its thread is done.');
    expect(demandSentence(one([], items, map))).toBe('No thread yet.');
  });

  it('lists its threads with what waits on the person first and what is done last', () => {
    const row = one(all, items, map);
    expect(threadsInOrder(row).map((t) => t.item.id)).toEqual(['w-needs', 'w-agent', 'w-start', 'w-later', 'w-done-new', 'w-done-old']);
    expect(threadsInOrder(row).map(wordForPlace)).toEqual(['Needs you', 'Working', 'To start', 'Later', 'Done', 'Done']);
  });

  it('says what was delivered in the demand\'s own words when it has them', () => {
    const row = one(all, items, map, { delivered: ['Refund email rewritten', 'Template shipped'] });
    expect(deliveredLines(row)).toEqual(['Refund email rewritten', 'Template shipped']);
  });

  it('falls back to the names of the threads that are done, oldest first', () => {
    expect(deliveredLines(one(all, items, map))).toEqual(['Refund copy', 'Refund template']);
    expect(deliveredLines(one(['w-needs'], items, map))).toEqual([]);
  });
});

describe('a thread\'s demand', () => {
  it('is found by project and id, so two projects\' threads never share one', () => {
    const by = demandByThread([demand('d-1', ['w-a']), { ...demand('d-2', ['w-a']), product: 'studio', productName: 'Studio' }] as never);
    expect(by.get('shop:w-a')!.id).toBe('d-1');
    expect(by.get('studio:w-a')!.id).toBe('d-2');
    expect(by.get('shop:w-b')).toBeUndefined();
  });
});
