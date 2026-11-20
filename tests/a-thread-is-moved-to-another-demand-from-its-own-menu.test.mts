// A THREAD IS MOVED TO ANOTHER DEMAND FROM ITS OWN MENU.
//
// A new thread is put in a demand without being asked (main/demand-sorter.mts),
// and a guess is sometimes wrong. The person has to be able to say so where
// they notice it, which is on the thread: its More actions menu has a Demand
// row under Priority that says the demand it is in and opens the project's
// demands with No demand first, the same shape as the Priority row. What the
// person picks there is theirs, so the sorter never moves the thread again
// (main/demands.mts: a person's placing outranks an agent's, and "none" is a
// placing).
//
// The new thread card offers the same demands in the same order, so the two
// menus cannot drift apart.
import { describe, it, expect } from 'vitest';
import { readFileSync } from 'node:fs';
import { demandChoices, type DemandRecord } from '../renderer/src/demands/demand-rules';
import { demandPlaces, threadMenuRows } from '../renderer/src/threads/ThreadMenu.tsx';

const demand = (id: string, more: Partial<DemandRecord> = {}): DemandRecord => ({
  id, product: 'shop', productName: 'Shop', name: `Demand ${id}`, threads: [], createdAt: 1, updatedAt: 1, ...more,
});
const ids = (list: readonly { id: string | null }[]) => list.map((d) => d.id);
const read = (file: string) => readFileSync(new URL(`../renderer/src/${file}`, import.meta.url), 'utf8');
const rows = (opts: Partial<Parameters<typeof threadMenuRows>[0]>) => threadMenuRows({ change: false, terminal: 'closed', finish: true, ...opts });

describe('the demands a thread can be moved to', () => {
  it('are its own project\'s, and never one that was removed', () => {
    const all = [demand('d-1'), demand('d-2', { product: 'blog' }), demand('d-3', { endedAt: 5 })];
    expect(ids(demandChoices(all, 'shop'))).toEqual(['d-1']);
  });

  it('list the ones still open first, each group newest first', () => {
    const all = [
      demand('d-old', { updatedAt: 10 }),
      demand('d-delivered', { updatedAt: 40, outcome: 'Refund emails go out by store.' }),
      demand('d-new', { updatedAt: 30 }),
      demand('d-delivered-older', { updatedAt: 20, outcome: 'The price list is live.' }),
    ];
    expect(ids(demandChoices(all, 'shop'))).toEqual(['d-new', 'd-old', 'd-delivered', 'd-delivered-older']);
  });

  it('are none for a project with no demand', () => {
    expect(demandChoices([demand('d-1')], 'blog')).toEqual([]);
  });
});

describe('the More actions menu', () => {
  it('has a Demand row saying the demand the thread is in', () => {
    const row = rows({ demand: { name: 'Refund emails by store' } }).find((r) => r.id === 'demand');
    expect(row).toMatchObject({ label: 'Demand', detail: 'Refund emails by store' });
  });

  it('says None on a thread that is in no demand', () => {
    expect(rows({ demand: { name: null } }).find((r) => r.id === 'demand')).toMatchObject({ detail: 'None' });
  });

  it('sits under Priority and above Pin', () => {
    expect(rows({ priority: 'high', demand: { name: null }, pin: 'unpinned' }).map((r) => r.id)).toEqual(['terminal', 'priority', 'demand', 'pin', 'done']);
  });

  it('has none where the project has no demand to pick', () => {
    expect(rows({ demand: null }).map((r) => r.id)).not.toContain('demand');
  });
});

describe('the demands the row opens', () => {
  const choices = [{ id: 'd-1', name: 'Refund emails by store' }, { id: 'd-2', name: 'Price list' }];

  it('start with No demand, and mark the one the thread is in', () => {
    const places = demandPlaces(choices, 'd-2');
    expect(places.map((p) => p.name)).toEqual(['No demand', 'Refund emails by store', 'Price list']);
    expect(ids(places.filter((p) => p.on))).toEqual(['d-2']);
  });

  it('mark No demand on a thread that is in none', () => {
    expect(ids(demandPlaces(choices, null).filter((p) => p.on))).toEqual([null]);
  });

  it('picking the one it is already in writes nothing', () => {
    expect(read('threads/ThreadMenu.tsx')).toContain('onClick={() => { close(false); if (!p.on) onDemand?.(p.id); }}');
  });
});

describe('the thread\'s page', () => {
  it('gives the menu its project\'s demands and places the thread where the person says', () => {
    const focus = read('components/Focus.tsx');
    expect(focus).toContain('demandChoices(placing.demands, item.product)');
    expect(focus).toMatch(/onDemand=\{[^}]*placing\?\.place\(item, /);
  });

  it('is given the demands and the way to place a thread by the app', () => {
    const app = read('App.tsx');
    expect(app).toContain('<DemandPlacingContext.Provider value={demandPlacing}>');
    expect(app).toMatch(/api\.placeInDemand\(\{ product: item\.product, thread: item\.id, demand \}\)/);
  });

  it('shares its list with the new thread card', () => {
    expect(read('threads/ThreadComposer.tsx')).toContain('demandChoices(demands ?? [], product.slug)');
  });
});
