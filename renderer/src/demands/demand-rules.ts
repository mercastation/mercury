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

/** The demands a thread of this project can be put in: the open ones first, each group newest first. */
export function demandChoices(demands: readonly DemandRecord[], product: string): DemandRecord[] {
  return demands
    .filter((d) => d.product === product && !d.endedAt)
    .sort((a, b) => Number(!!a.outcome) - Number(!!b.outcome) || b.updatedAt - a.updatedAt);
}

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

/* ------------------------------ a demand's page ----------------------------- */

const ORDER: (Place | null)[] = ['needs', 'progress', 'later', null, 'done'];

/** Its threads, what waits on the person first and what is done last. */
export function threadsInOrder(row: DemandRow): DemandThread[] {
  const rank = (t: DemandThread) => ORDER.indexOf(t.place) * 2 + (t.place === 'later' && !t.toStart ? 1 : 0);
  return [...row.threads].sort((a, b) => rank(a) - rank(b) || (b.item.updatedAt ?? 0) - (a.item.updatedAt ?? 0));
}

export function wordForPlace(t: DemandThread): string {
  if (t.place === 'needs') return 'Needs you';
  if (t.place === 'progress') return 'Working';
  if (t.place === 'later') return t.toStart ? 'To start' : 'Later';
  return t.place === 'done' ? 'Done' : 'Open';
}

const LINES_SHOWN = 6;

/** What was delivered: the demand's own words, else the threads that are done, oldest first. */
export function deliveredLines(row: DemandRow): string[] {
  if (row.demand.delivered?.length) return row.demand.delivered;
  return row.threads.filter((t) => t.place === 'done').sort((a, b) => (a.item.updatedAt ?? 0) - (b.item.updatedAt ?? 0)).map((t) => t.name).slice(-LINES_SHOWN);
}

/** What waits on the person, then how far along the demand is. */
export function demandSentence(row: DemandRow): string {
  if (!row.total) return 'No thread yet.';
  if (row.state === 'delivered') return row.total === 1 ? 'Its thread is done.' : `All ${row.total} threads are done.`;
  const at = (place: Place) => row.threads.filter((t) => t.place === place);
  const [needs, working, toStart] = [at('needs').length, at('progress').length, row.threads.filter((t) => t.toStart).length];
  const lead = needs ? (needs === 1 ? '1 thread needs you' : `${needs} threads need you`)
    : working ? (working === 1 ? 'An agent is on 1 thread' : `Agents are on ${working} threads`)
    : toStart ? (toStart === 1 ? '1 thread waits to start' : `${toStart} threads wait to start`)
    : 'Waiting in Later';
  return `${lead}. ${row.done} of ${row.total} ${row.done === 1 || row.total === 1 ? 'is' : 'are'} done.`;
}

export interface DemandTabs { today: DemandRow[]; progress: DemandRow[]; todo: DemandRow[]; delivered: DemandRow[] }
export type DemandTabId = keyof DemandTabs;

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
