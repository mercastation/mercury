// A DEMAND: the piece of work a person wants delivered, which threads serve.
//
// Kept like a repeating task (shared/repeats.mts): a record of its own in
// <product>/demands.jsonl, never a work item, so no agent is ever started on
// one and no thread list has to leave it out. The file holds two kinds of
// line, both folded newest-wins:
//
//   { id, ts, patch }                  a demand's own fields
//   { thread, ts, demand, source }     the demand a thread serves; null says
//                                      "none", so the app does not ask again
//
// Pure, for the main process and the renderer alike.
import { dateKey } from './repeats.mts';

export { dateKey as dayKey };

export interface Demand {
  id: string;
  /** The project it belongs to, set where demands of several are listed together. */
  product?: string;
  name: string;
  /** What it is, in a sentence or two. */
  about?: string;
  /** What has been delivered so far, oldest first. */
  delivered?: string[];
  /** What was delivered, once every thread is done. */
  outcome?: string;
  /** The day it was picked for, as a day key. */
  today?: string;
  /** When its report (about, delivered, outcome) was last written. */
  wroteAt?: number;
  endedAt?: number;
  createdAt: number;
  updatedAt: number;
  /** The threads that serve it, in the order they joined. */
  threads: string[];
}

export type PlacedBy = 'person' | 'agent';
export interface DemandLine { id: string; ts: number; patch: Record<string, unknown> }
export interface PlaceLine { thread: string; ts: number; demand: string | null; source?: PlacedBy }
export interface Placement { demand: string | null; source: PlacedBy; ts: number }
export interface Demands {
  demands: Map<string, Demand>;
  /** Every thread anyone placed, in a demand or in none. */
  placed: Map<string, Placement>;
}

/** One line of a phone at the list's type size. */
export const DEMAND_NAME_MAX = 60;
const TEXT_MAX = 400;
const LINE_MAX = 160;
const LINES_MAX = 6;
const DAY = /^\d{4}-\d{2}-\d{2}$/;

const words = (v: unknown, max: number) => (typeof v === 'string' ? v.replace(/\s+/g, ' ').trim().slice(0, max) : '');
const moment = (v: unknown) => (typeof v === 'number' && Number.isFinite(v) && v > 0 ? v : undefined);

/** A field's value as a demand keeps it, or `undefined` for one it does not take. `null` clears. */
const FIELDS: Record<string, (v: unknown) => unknown> = {
  name: (v) => words(v, DEMAND_NAME_MAX) || undefined,
  about: (v) => (v === '' ? null : words(v, TEXT_MAX) || undefined),
  delivered: (v) => (Array.isArray(v) ? v.map((l) => words(l, LINE_MAX)).filter(Boolean).slice(-LINES_MAX) : undefined),
  outcome: (v) => (v === '' ? null : words(v, TEXT_MAX) || undefined),
  today: (v) => (v === '' ? null : typeof v === 'string' && DAY.test(v) ? v : undefined),
  wroteAt: moment,
  endedAt: moment,
};
export const DEMAND_FIELDS = Object.keys(FIELDS);

type Folding = Demand & { _ts: Record<string, number> };

export function foldDemands(lines: ReadonlyArray<unknown>): Demands {
  const folding = new Map<string, Folding>();
  const placed = new Map<string, Placement>();
  for (const line of lines) {
    if (!line || typeof line !== 'object') continue;
    const { id, thread, ts, patch, demand, source } = line as Partial<DemandLine & PlaceLine>;
    if (typeof ts !== 'number' || !Number.isFinite(ts)) continue;
    if (typeof thread === 'string' && thread) {
      if (demand !== null && typeof demand !== 'string') continue;
      const by: PlacedBy = source === 'person' ? 'person' : 'agent';
      const held = placed.get(thread);
      // The person's choice stands until the person changes it.
      if (held && ((held.source === 'person' && by === 'agent') || (held.source === by && ts < held.ts))) continue;
      placed.set(thread, { demand, source: by, ts });
      continue;
    }
    if (typeof id !== 'string' || !id || !patch || typeof patch !== 'object') continue;
    let d = folding.get(id);
    if (!d) {
      d = { id, name: '', createdAt: ts, updatedAt: ts, threads: [], _ts: {} };
      folding.set(id, d);
    }
    for (const field of DEMAND_FIELDS) {
      if (!(field in patch)) continue;
      const value = FIELDS[field]((patch as Record<string, unknown>)[field]);
      if (value === undefined || (d._ts[field] !== undefined && ts < d._ts[field])) continue;
      if (value === null) delete (d as unknown as Record<string, unknown>)[field];
      else (d as unknown as Record<string, unknown>)[field] = value;
      d._ts[field] = ts;
    }
    d.createdAt = Math.min(d.createdAt, ts);
    d.updatedAt = Math.max(d.updatedAt, ts);
  }
  const demands = new Map<string, Demand>();
  for (const { _ts, ...d } of folding.values()) if (d.name) demands.set(d.id, d);
  for (const [thread, at] of [...placed].sort((a, b) => a[1].ts - b[1].ts)) {
    const d = at.demand ? demands.get(at.demand) : null;
    if (d && !d.endedAt) d.threads.push(thread);
  }
  return { demands, placed };
}

/** The demand a thread serves, or null: never placed, placed in none, or its demand has ended. */
export function demandOf(thread: string, { demands, placed }: Demands): Demand | null {
  const id = placed.get(thread)?.demand;
  const d = id ? demands.get(id) : null;
  return d && !d.endedAt ? d : null;
}

export function pickedForToday(demand: Pick<Demand, 'today'>, now = Date.now()): boolean {
  return !!demand.today && demand.today === dateKey(now);
}
