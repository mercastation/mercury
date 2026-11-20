// A DEMAND IS A RECORD OF ITS OWN THAT THREADS SERVE.
//
// What was missing, measured on one person's store on 2026-11-20: 97 threads
// in 18 days, 30 finished in the last week, and nothing above a thread to say
// which piece of work each one served. The person kept that map in their head.
//
// A demand is that piece of work. It is kept like a repeating task, in a file
// of its own beside the threads' ledger, so no agent can ever be started on
// one. The file holds a demand's own fields and, line by line, which demand
// each thread serves. Covered here: the fold of both kinds of line, a thread
// serving one demand at a time, "none" said out loud so nobody is asked twice,
// the person's choice outranking the app's, an ended demand letting go of its
// threads, and the day a demand is picked for.
import { describe, it, expect } from 'vitest';
import { foldDemands, demandOf, pickedForToday, dayKey, DEMAND_NAME_MAX } from '../shared/demands.mts';

const T0 = 1_795_000_000_000;
const MIN = 60_000;
const demand = (id: string, patch: Record<string, unknown>, ts = T0) => ({ id, ts, patch });
const place = (thread: string, to: string | null, ts = T0, source: 'person' | 'agent' = 'agent') => ({ thread, ts, demand: to, source });

describe('a demand, folded from its lines', () => {
  it('keeps the newest value of each field', () => {
    const { demands } = foldDemands([
      demand('d-1', { name: 'Stock reservation' }),
      demand('d-1', { about: 'The store reserves stock before the order exists.' }, T0 + MIN),
      demand('d-1', { name: 'Stock reservation for pickup' }, T0 + 2 * MIN),
    ]);
    expect(demands.get('d-1')).toMatchObject({
      id: 'd-1', name: 'Stock reservation for pickup', about: 'The store reserves stock before the order exists.',
      createdAt: T0, updatedAt: T0 + 2 * MIN, threads: [],
    });
  });

  it('is not moved back by a line that arrives late', () => {
    const { demands } = foldDemands([demand('d-1', { name: 'New name' }, T0 + MIN), demand('d-1', { name: 'Old name' }, T0)]);
    expect(demands.get('d-1')!.name).toBe('New name');
  });

  it('drops what is not a demand: a torn line, a field nobody defined, a name that is not words', () => {
    const { demands } = foldDemands([
      null, 'not a line' as never, { id: 'd-1', ts: 'soon', patch: { name: 'x' } } as never,
      demand('d-2', { name: 'Kept', colour: 'red', delivered: ['One', 7, 'Two'] }),
      demand('d-3', { name: '   ' }),
    ]);
    expect([...demands.keys()]).toEqual(['d-2']);
    expect(demands.get('d-2')).not.toHaveProperty('colour');
    expect(demands.get('d-2')!.delivered).toEqual(['One', 'Two']);
  });

  it('holds a name to one line of a phone', () => {
    const { demands } = foldDemands([demand('d-1', { name: 'x'.repeat(DEMAND_NAME_MAX + 40) })]);
    expect(demands.get('d-1')!.name).toHaveLength(DEMAND_NAME_MAX);
  });
});

describe('which demand a thread serves', () => {
  const lines = [demand('d-1', { name: 'Checkout' }), demand('d-2', { name: 'Pricing page' })];

  it('is the last one it was put in, and only that one', () => {
    const folded = foldDemands([...lines, place('w-a', 'd-1'), place('w-b', 'd-1'), place('w-a', 'd-2', T0 + MIN)]);
    expect(folded.demands.get('d-1')!.threads).toEqual(['w-b']);
    expect(folded.demands.get('d-2')!.threads).toEqual(['w-a']);
    expect(demandOf('w-a', folded)!.id).toBe('d-2');
  });

  it('is none when nobody placed it, and none when it was said to serve none', () => {
    const folded = foldDemands([...lines, place('w-a', null)]);
    expect(demandOf('w-a', folded)).toBeNull();
    expect(demandOf('w-never', folded)).toBeNull();
    expect(folded.placed.has('w-a')).toBe(true);
    expect(folded.placed.has('w-never')).toBe(false);
  });

  it('stays where the person put it, whatever the app says later', () => {
    const folded = foldDemands([...lines, place('w-a', 'd-1', T0, 'person'), place('w-a', 'd-2', T0 + MIN, 'agent')]);
    expect(demandOf('w-a', folded)!.id).toBe('d-1');
  });

  it('moves when the person moves it, even after the app placed it', () => {
    const folded = foldDemands([...lines, place('w-a', 'd-1', T0, 'agent'), place('w-a', 'd-2', T0 + MIN, 'person')]);
    expect(demandOf('w-a', folded)!.id).toBe('d-2');
  });

  it('is none for a demand that does not exist or has ended', () => {
    const folded = foldDemands([...lines, demand('d-2', { endedAt: T0 + MIN }, T0 + MIN), place('w-a', 'd-2'), place('w-b', 'd-gone')]);
    expect(demandOf('w-a', folded)).toBeNull();
    expect(demandOf('w-b', folded)).toBeNull();
  });

  it('lists a demand\'s threads in the order they joined', () => {
    const folded = foldDemands([...lines, place('w-c', 'd-1', T0 + 2 * MIN), place('w-a', 'd-1', T0), place('w-b', 'd-1', T0 + MIN)]);
    expect(folded.demands.get('d-1')!.threads).toEqual(['w-a', 'w-b', 'w-c']);
  });
});

describe('the day a demand is picked for', () => {
  const noon = new Date(2026, 10, 20, 12, 0, 0).getTime();
  const DAY = 24 * 60 * MIN;

  it('is that day and no other', () => {
    const { demands } = foldDemands([demand('d-1', { name: 'Checkout', today: dayKey(noon) })]);
    const d = demands.get('d-1')!;
    expect(pickedForToday(d, noon)).toBe(true);
    expect(pickedForToday(d, noon + DAY)).toBe(false);
    expect(pickedForToday(d, noon - DAY)).toBe(false);
  });

  it('is taken back by an empty day', () => {
    const { demands } = foldDemands([demand('d-1', { name: 'Checkout', today: dayKey(noon) }), demand('d-1', { today: '' }, T0 + MIN)]);
    expect(pickedForToday(demands.get('d-1')!, noon)).toBe(false);
  });

  it('is never a day that is not a date', () => {
    const { demands } = foldDemands([demand('d-1', { name: 'Checkout', today: 'tomorrow' })]);
    expect(demands.get('d-1')!.today).toBeUndefined();
  });
});
