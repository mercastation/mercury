// A DEMAND IS KEPT BESIDE THE THREADS, AND IS NEVER ONE OF THEM.
//
// A demand groups threads. Kept as a work item it would be one more open row,
// and an open row is what the app starts agents on, lists in Needs you and
// names. So it is written to a file of its own, the way a repeating task is,
// and the threads' ledger never hears of it.
//
// Covered here: making one writes no work item; a thread is put in one demand
// and moved; the app's guess gives way to the person's choice; a name is
// required; ending one lets its threads go; and the day it is picked for.
import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { DemandStore } from '../main/demands.mts';
import { machineryPath } from '../main/store/home.mts';
import { dayKey } from '../shared/demands.mts';

const T0 = 1_795_000_000_000;
const MIN = 60_000;

let dir: string;
let demands: DemandStore;
beforeEach(() => {
  dir = fs.mkdtempSync(path.join(os.tmpdir(), 'demands-'));
  fs.writeFileSync(path.join(dir, 'project.json'), JSON.stringify({ schemaVersion: 1, id: 'p', name: 'P' }));
  demands = new DemandStore();
});
afterEach(() => { fs.rmSync(dir, { recursive: true, force: true }); });

describe('making a demand', () => {
  it('writes it to its own file and no work item anywhere', () => {
    const made = demands.make(dir, { name: 'Stock reservation', about: 'Reserve before the order exists.' }, T0);
    expect(made).toMatchObject({ name: 'Stock reservation', about: 'Reserve before the order exists.', threads: [], createdAt: T0 });
    expect(made.id).toMatch(/^d-[0-9a-f]{10}$/);
    expect(fs.existsSync(machineryPath(dir, 'demands.jsonl'))).toBe(true);
    expect(fs.existsSync(machineryPath(dir, 'work-items.jsonl'))).toBe(false);
  });

  it('needs a name', () => {
    expect(() => demands.make(dir, { name: '   ' }, T0)).toThrow(/needs a name/);
    expect(demands.list(dir)).toEqual([]);
  });

  it('lists what was made, oldest first, without the ended ones', () => {
    const a = demands.make(dir, { name: 'Checkout' }, T0);
    const b = demands.make(dir, { name: 'Pricing page' }, T0 + MIN);
    demands.change(dir, a.id, { endedAt: T0 + 2 * MIN }, T0 + 2 * MIN);
    expect(demands.list(dir).map((d) => d.id)).toEqual([b.id]);
  });
});

describe('putting a thread in a demand', () => {
  it('puts it in one, and moving it takes it out of the other', () => {
    const a = demands.make(dir, { name: 'Checkout' }, T0);
    const b = demands.make(dir, { name: 'Pricing page' }, T0);
    demands.place(dir, 'w-1', a.id, { source: 'person' }, T0 + MIN);
    demands.place(dir, 'w-1', b.id, { source: 'person' }, T0 + 2 * MIN);
    expect(demands.get(dir, a.id)!.threads).toEqual([]);
    expect(demands.get(dir, b.id)!.threads).toEqual(['w-1']);
    expect(demands.of(dir, 'w-1')!.id).toBe(b.id);
  });

  it('keeps the person\'s choice over the app\'s later guess', () => {
    const a = demands.make(dir, { name: 'Checkout' }, T0);
    const b = demands.make(dir, { name: 'Pricing page' }, T0);
    demands.place(dir, 'w-1', a.id, { source: 'person' }, T0 + MIN);
    demands.place(dir, 'w-1', b.id, { source: 'agent' }, T0 + 2 * MIN);
    expect(demands.of(dir, 'w-1')!.id).toBe(a.id);
  });

  it('remembers a thread that serves none, so it is not asked about again', () => {
    demands.make(dir, { name: 'Checkout' }, T0);
    expect(demands.isPlaced(dir, 'w-1')).toBe(false);
    demands.place(dir, 'w-1', null, { source: 'agent' }, T0 + MIN);
    expect(demands.isPlaced(dir, 'w-1')).toBe(true);
    expect(demands.of(dir, 'w-1')).toBeNull();
  });

  it('refuses a demand that is not there', () => {
    expect(() => demands.place(dir, 'w-1', 'd-0000000000', { source: 'person' }, T0)).toThrow(/no such demand/);
  });
});

describe('changing a demand', () => {
  it('renames it, picks it for a day and takes the day back', () => {
    const a = demands.make(dir, { name: 'Checkout' }, T0);
    const day = dayKey(T0);
    expect(demands.change(dir, a.id, { name: 'Checkout on the phone', today: day }, T0 + MIN)).toMatchObject({ name: 'Checkout on the phone', today: day });
    expect(demands.change(dir, a.id, { today: '' }, T0 + 2 * MIN)!.today).toBeUndefined();
  });

  it('takes only what a demand is made of', () => {
    const a = demands.make(dir, { name: 'Checkout' }, T0);
    expect(() => demands.change(dir, a.id, { status: 'done' }, T0 + MIN)).toThrow(/nothing a demand keeps/);
    expect(() => demands.change(dir, 'd-0000000000', { name: 'x' }, T0 + MIN)).toThrow(/no such demand/);
  });

  it('ended, lets its threads go', () => {
    const a = demands.make(dir, { name: 'Checkout' }, T0);
    demands.place(dir, 'w-1', a.id, { source: 'person' }, T0 + MIN);
    demands.change(dir, a.id, { endedAt: T0 + 2 * MIN }, T0 + 2 * MIN);
    expect(demands.of(dir, 'w-1')).toBeNull();
  });
});
