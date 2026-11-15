// A RUN LEFT GOING WHEN SHE WALKS AWAY MOVES TO THE CLOUD (2026-11-15).
//
// An agent started at 02:28 on a Mac left open stopped when the Mac fell
// asleep and stayed stopped for nine hours: 27 "Connection lost while your
// computer was asleep" retries in its transcript, and 535 minutes on the row.
// The cloud was set up and had room, but under "when this Mac is full" a run
// stays where it started. Measured over that fortnight's runs: 18 of 110
// hours went to the Mac being asleep. Her words: "I thought the work would go
// to the cloud then."
//
// Now ten minutes with no keyboard or mouse on a Mac that is awake count as
// away, and away "when this Mac is full" behaves as Everything: each run the
// cloud can take moves there, and new ones start there. Back at the Mac, her
// setting holds again.
import { describe, it, expect, afterEach } from 'vitest';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { Supervisor } from '../main/supervisor.mts';

const MIN = 60_000;
const dirs: string[] = [];
afterEach(() => { while (dirs.length) fs.rmSync(dirs.pop()!, { recursive: true, force: true }); });

const folder = (id: string) => `/r/.claude/worktrees/${id}`;

type Row = { id: string; product: string; status: string };
function make({ cloudRuns = null as unknown, idle = 0 as number | null } = {}) {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'away-to-cloud-'));
  dirs.push(root);
  const product = { slug: 'p', dir: root, repoPath: '/r' };
  const rows: Row[] = [];
  const sup = new Supervisor(
    { storeRoot: root, home: root, maxConcurrentSessions: 6, maxCloudSessions: 10, authProfiles: ['default'], cloudRuns },
    { listItems: () => rows, listProducts: () => [product], isDue: () => true },
    root,
  );
  sup._cloudSetUp = () => true;
  sup._cloudReady = () => true;
  sup._cloudSignedInAtAll = () => true;
  sup._cloudVersionGaps = () => [];
  sup.transcriptFile = (rec: { sessionId?: string } | null) => (rec?.sessionId ? '/t.jsonl' : null);
  let idleMs = idle;
  if (idle !== null) sup.idleMs = () => idleMs;
  const killed: string[] = [];
  const run = (id: string) => {
    rows.push({ id, product: 'p', status: 'claimed' });
    const s = { itemId: id, product: 'p', cwd: folder(id), engine: 'claude', startedAt: Date.now(), tail: [], child: { kill: () => killed.push(id) } };
    sup.sessions.set(id, s);
    sup._liveSessions[id] = { product: 'p', sessionId: `s-${id}`, cwd: s.cwd, engine: s.engine };
    return s;
  };
  return { sup, rows, product, run, killed, setIdle: (ms: number) => { idleMs = ms; } };
}

describe('under "when this Mac is full"', () => {
  it('moves a run to the cloud once nobody has touched the Mac for ten minutes (the reported case)', () => {
    const { sup, rows, run, killed } = make({ idle: 10 * MIN });
    const s = run('w-a');
    sup._moveRunsToTheCloud(rows);
    expect(killed).toEqual(['w-a']);
    // Paused the way a move pauses one, so the sweep resumes the same session there.
    expect(s.stoppedByUs).toBe(true);
    expect(sup._recoverPending).toBe(true);
  });

  it('leaves it on the Mac while she is there, nine minutes idle included', () => {
    for (const idle of [0, 9 * MIN]) {
      const { sup, rows, run, killed } = make({ idle });
      run('w-a');
      sup._moveRunsToTheCloud(rows);
      expect(killed, String(idle)).toEqual([]);
    }
  });

  it('starts a new run in the cloud while she is away, and on the Mac once she is back', () => {
    const { sup, rows, product, setIdle } = make({ idle: 30 * MIN });
    rows.push({ id: 'w-new', product: 'p', status: 'open' });
    sup._cloudSlotFor = () => true;
    sup._localSlotFor = () => true;
    const where = () => sup._whereToRun({ item: rows[0], product, cwd: folder('w-new'), engine: 'claude' });
    expect(where()).toBe('cloud');
    setIdle(5_000);
    expect(where()).toBe('local');
  });

  it('moves nothing in one of a shut Mac\'s few-second wakes, when nothing may start', () => {
    const { sup, rows, run, killed } = make({ idle: 60 * MIN });
    run('w-a');
    sup.asleep = () => true;
    sup._moveRunsToTheCloud(rows);
    expect(killed).toEqual([]);
  });
});

describe('what being away does not change', () => {
  it('never sends anything with the cloud turned off', () => {
    const { sup, rows, run, killed } = make({ cloudRuns: false, idle: 60 * MIN });
    run('w-a');
    sup._moveRunsToTheCloud(rows);
    expect(killed).toEqual([]);
    expect(sup._cloudFirst()).toBe(false);
  });

  it('is never true on a copy with no keyboard to read, as a headless one', () => {
    const { sup, rows, run, killed } = make({ idle: null });
    run('w-a');
    sup._moveRunsToTheCloud(rows);
    expect(killed).toEqual([]);
  });

  it('keeps Everything as Everything, at the Mac or away', () => {
    for (const idle of [0, 60 * MIN]) expect(make({ cloudRuns: 'all', idle }).sup._cloudFirst(), String(idle)).toBe(true);
  });
});
