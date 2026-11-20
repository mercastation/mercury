// A THREAD IN A DEMAND LISTS ONLY WHAT WAITS ON YOU.
//
// A thread used to be the only place its filed threads could be found, so its
// page listed every one of them, running and finished included: threads inside
// a thread. With demands the work is one level deep. A thread another thread
// filed joins that thread's demand, and the demand's page is where all of them
// are listed with their states.
//
// What stays on the thread is the part that is a question to the person: the
// proposals waiting for a yes, with Dismiss, Do here and New thread. A thread
// with no demand keeps the whole list, because nothing else would show it.
import { describe, it, expect } from 'vitest';
import { readFileSync } from 'node:fs';
import { madeShown } from '../renderer/src/threads-made';

type Row = { id: string; approve?: boolean };
const canPress = (r: Row) => !!r.approve;
const rows: Row[] = [{ id: 'w-running' }, { id: 'w-ask-1', approve: true }, { id: 'w-done' }, { id: 'w-ask-2', approve: true }];
const ids = (list: Row[]) => list.map((r) => r.id);

describe('the threads a thread\'s page lists', () => {
  it('are, in a demand, only the proposals waiting for a yes', () => {
    expect(ids(madeShown(rows, canPress, { inDemand: true }))).toEqual(['w-ask-1', 'w-ask-2']);
  });

  it('are none at all in a demand when nothing waits', () => {
    expect(madeShown([{ id: 'w-running' }, { id: 'w-done' }], canPress, { inDemand: true })).toEqual([]);
  });

  it('are every one of them outside a demand, what waits first', () => {
    expect(ids(madeShown(rows, canPress))).toEqual(['w-ask-1', 'w-ask-2', 'w-running', 'w-done']);
    expect(ids(madeShown([{ id: 'w-running' }, { id: 'w-done' }], canPress))).toEqual(['w-running', 'w-done']);
  });
});

describe('the list on a thread\'s page', () => {
  const read = (file: string) => readFileSync(new URL(`../renderer/src/${file}`, import.meta.url), 'utf8');

  it('draws what that rule gives it, and nothing when that is nothing', () => {
    const source = read('components/ThreadsMade.tsx');
    expect(source).toContain('madeShown(rows, canPress, { inDemand })');
    expect(source).toContain('if (!shown.length) return null;');
  });

  it('is told whether its thread serves a demand, on both pages that draw a thread', () => {
    expect(read('components/ItemThread.tsx')).toMatch(/<ThreadsMade[^>]*inDemand=\{/);
    expect(read('components/Focus.tsx')).toMatch(/<ThreadsMade[\s\S]{0,400}inDemand=\{/);
  });
});
