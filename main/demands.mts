// The demand store: the only writer of <product>/demands.jsonl.
//
// A demand is never a work item (shared/demands.mts says why), so nothing here
// touches the threads' ledger. The file is append-only, folded newest-wins and
// read whole, like repeats.jsonl.
import crypto from 'node:crypto';
import fs from 'node:fs';
import { DEMAND_FIELDS, demandOf, foldDemands, type Demand, type Demands, type PlacedBy } from '../shared/demands.mts';
import { machineryPath } from './store/home.mts';

const FILE = 'demands.jsonl';

export interface DemandInput { name: string; about?: string; today?: string }

export class DemandStore {
  filePath(productDir: string): string {
    return machineryPath(productDir, FILE);
  }

  _read(productDir: string): unknown[] {
    let raw = '';
    try {
      raw = fs.readFileSync(this.filePath(productDir), 'utf8');
    } catch (err) {
      if ((err as NodeJS.ErrnoException).code === 'ENOENT') return [];
      throw err;
    }
    return raw.split('\n').filter(Boolean).map((line) => {
      try { return JSON.parse(line); } catch { return null; }
    });
  }

  _append(productDir: string, line: Record<string, unknown>): void {
    fs.appendFileSync(this.filePath(productDir), `${JSON.stringify(line)}\n`);
  }

  folded(productDir: string): Demands {
    return foldDemands(this._read(productDir));
  }

  /** The demands still going, oldest first. */
  list(productDir: string): Demand[] {
    return [...this.folded(productDir).demands.values()].filter((d) => !d.endedAt).sort((a, b) => a.createdAt - b.createdAt);
  }

  get(productDir: string, id: string): Demand | null {
    return this.folded(productDir).demands.get(id) ?? null;
  }

  /** The demand a thread serves, or null. */
  of(productDir: string, thread: string): Demand | null {
    return demandOf(thread, this.folded(productDir));
  }

  /** Whether anyone has said where this thread belongs, a demand or none. */
  isPlaced(productDir: string, thread: string): boolean {
    return this.folded(productDir).placed.has(thread);
  }

  make(productDir: string, { name, about, today }: DemandInput, now = Date.now()): Demand {
    if (!String(name ?? '').trim()) throw new Error('a demand needs a name');
    const id = `d-${crypto.randomBytes(5).toString('hex')}`;
    this._append(productDir, { id, ts: now, patch: { name, ...(about ? { about } : {}), ...(today ? { today } : {}) } });
    return this.get(productDir, id)!;
  }

  change(productDir: string, id: string, patch: Record<string, unknown>, now = Date.now()): Demand | null {
    if (!this.get(productDir, id)) throw new Error(`no such demand: ${id}`);
    const kept = Object.fromEntries(Object.entries(patch).filter(([field]) => DEMAND_FIELDS.includes(field)));
    if (!Object.keys(kept).length) throw new Error('nothing a demand keeps was in that change');
    this._append(productDir, { id, ts: now, patch: kept });
    return this.get(productDir, id);
  }

  /** Put a thread in a demand, or in none (`null`). */
  place(productDir: string, thread: string, demand: string | null, { source = 'agent' }: { source?: PlacedBy } = {}, now = Date.now()): void {
    if (!thread) throw new Error('a thread is needed to place it');
    if (demand !== null) {
      const d = this.get(productDir, demand);
      if (!d || d.endedAt) throw new Error(`no such demand: ${demand}`);
    }
    this._append(productDir, { thread, ts: now, demand, source });
  }
}
