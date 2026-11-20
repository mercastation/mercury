// Where a demand stands, read off its threads (shared/demands.mts says what a
// demand is). Nothing here is typed by the person: a thread's place is the tab
// the inbox already has it under, and a demand's state follows from those.
import { pickedForToday, type Demand } from '../../../shared/demands.mts';
import { isProposal, rowTitle } from '../list-rules';
import type { WorkItem } from '../types';

export type DemandRecord = Demand & { product: string; productName: string };
/** What a person may change on a demand. An empty `today` takes the day back. */
export type DemandPatch = Partial<Pick<Demand, 'name' | 'about' | 'today' | 'endedAt'>>;

/** The inbox tab a thread is under, as the app's own lists have it. */
export type Place = 'needs' | 'progress' | 'later' | 'done';
export type DemandState = 'todo' | 'progress' | 'delivered';

export interface DemandThread {
  item: WorkItem;
  name: string;
  place: Place | null;
  /** In Later and waiting for the person to start it. */
  toStart: boolean;
}

export interface DemandRow {
  demand: DemandRecord;
  threads: DemandThread[];
  state: DemandState;
  today: boolean;
  total: number;
  done: number;
  /** Begun and not done: waiting on the person or with an agent. */
  open: number;
  later: number;
  /** A thread of it needs the person, or waits for the person to start it. */
  waitsOnYou: boolean;
  /** The state in a word or two, then what that word is about. */
  word: string;
  what: string;
  deliveredAt: number | null;
}

const threadKey = (product: string, id: string) => `${product}:${id}`;

/** Each thread's demand, by project and id. */
export function demandByThread(demands: readonly DemandRecord[]): Map<string, DemandRecord> {
  const by = new Map<string, DemandRecord>();
  for (const d of demands) for (const id of d.threads) by.set(threadKey(d.product, id), d);
  return by;
}

export const demandOfThread = (by: Map<string, DemandRecord>, i: { product: string; id: string }) => by.get(threadKey(i.product, i.id)) ?? null;

const andMore = (names: string[]) => (names.length > 1 ? `${names[0]}, and ${names.length - 1} more` : names[0] ?? '');
const newestStep = (progress: unknown) => String(progress ?? '').split('\n').map((l) => l.trim()).filter(Boolean).pop() ?? '';

function rowOf(demand: DemandRecord, items: Map<string, WorkItem>, where: (i: WorkItem) => Place | null, now: number): DemandRow {
  const threads: DemandThread[] = [];
  for (const id of demand.threads) {
    const item = items.get(threadKey(demand.product, id));
    if (!item) continue;
    const place = item.status === 'done' ? 'done' : where(item);
    threads.push({ item, name: rowTitle(item), place, toStart: place === 'later' && isProposal(item) });
  }
  const at = (place: Place) => threads.filter((t) => t.place === place);
  const [needs, working, later, done] = [at('needs'), at('progress'), at('later'), at('done')];
  const toStart = later.filter((t) => t.toStart);
  const total = threads.length;
  const state: DemandState = total > 0 && done.length === total ? 'delivered' : needs.length + working.length + done.length > 0 ? 'progress' : 'todo';
  const deliveredAt = state === 'delivered' ? Math.max(...done.map((t) => t.item.updatedAt ?? 0)) : null;
  let word = 'To do';
  let what = '';
  if (needs.length) {
    word = needs.length > 1 ? `${needs.length} need you` : 'Needs you';
    what = andMore(needs.map((t) => t.name));
  } else if (working.length) {
    word = 'Working';
    what = newestStep(working[0].item.progress) || working[0].name;
  } else if (toStart.length) {
    word = `${toStart.length} to start`;
    what = andMore(toStart.map((t) => t.name));
  } else if (later.length) {
    word = 'Later';
    what = andMore(later.map((t) => t.name));
  } else if (state === 'delivered') {
    word = 'Delivered';
    what = total === 1 ? '1 thread' : `${total} threads`;
  }
  return {
    demand, threads, state, today: pickedForToday(demand, now), total,
    done: done.length, open: needs.length + working.length, later: later.length,
    waitsOnYou: needs.length > 0 || (working.length === 0 && toStart.length > 0),
    word, what, deliveredAt,
  };
}

/** Every demand with where it stands. `where` is the inbox tab a thread is under. */
export function demandRows(demands: readonly DemandRecord[], items: readonly WorkItem[], where: (i: WorkItem) => Place | null, now = Date.now()): DemandRow[] {
  const byKey = new Map(items.map((i) => [threadKey(i.product, i.id), i]));
  return demands.map((d) => rowOf(d, byKey, where, now));
}

export interface DemandTabs { today: DemandRow[]; progress: DemandRow[]; todo: DemandRow[]; delivered: DemandRow[] }

const newest = (a: DemandRow, b: DemandRow) => (b.deliveredAt ?? b.demand.updatedAt) - (a.deliveredAt ?? a.demand.updatedAt);
/** What waits on the person first, then the newest. */
const byNeed = (a: DemandRow, b: DemandRow) => Number(b.waitsOnYou) - Number(a.waitsOnYou) || newest(a, b);

export function demandTabs(rows: readonly DemandRow[]): DemandTabs {
  return {
    today: rows.filter((r) => r.today).sort(byNeed),
    progress: rows.filter((r) => r.state === 'progress').sort(byNeed),
    todo: rows.filter((r) => r.state === 'todo').sort(byNeed),
    delivered: rows.filter((r) => r.state === 'delivered').sort(newest),
  };
}

const WEEK = 7 * 86_400_000;

/** Threads finished in the last seven days, by project, most first. */
export function finishedThisWeek(items: readonly WorkItem[], now = Date.now()): { total: number; byProject: { name: string; count: number }[] } {
  const counts = new Map<string, number>();
  for (const i of items) {
    if (i.status !== 'done' || now - (i.updatedAt ?? 0) >= WEEK) continue;
    const name = i.productName || i.product;
    counts.set(name, (counts.get(name) ?? 0) + 1);
  }
  const byProject = [...counts].map(([name, count]) => ({ name, count })).sort((a, b) => b.count - a.count || a.name.localeCompare(b.name));
  return { total: byProject.reduce((n, p) => n + p.count, 0), byProject };
}
