// A THREAD STARTS KNOWING ITS DEMAND AND WHAT ITS SIBLING THREADS DID.
//
// Why: threads filed under other threads are leaving the screen, and what the
// nesting gave an agent was the work around it. A demand gives the same, to
// every thread that serves it: the brief names the demand, says what it is,
// and says where each of its other threads stands, with the id to leave one a
// note by. An agent that knows a sibling already shipped the migration does
// not write it again.
//
// Covered here: the block for a thread with siblings, with none, and with no
// demand; how much of it there may be; and the app putting it in the brief.
import { describe, it, expect } from 'vitest';
import { mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { demandBriefBlock, SIBLINGS_SHOWN } from '../main/demand-brief.mts';
import { DemandStore } from '../main/demands.mts';
import { Supervisor } from '../main/supervisor.mts';

const NOW = 1_795_000_000_000;
const demand = { id: 'd-1', name: 'Refunds a customer can follow', about: 'The emails and the refund page promise the same thing.', threads: [], createdAt: NOW, updatedAt: NOW };
const sibling = (id: string, over: Record<string, unknown> = {}) => ({
  id, status: 'done', title: 'rewrite the refund email', label: 'Refund email rewrite',
  problem: 'The email promised five days and the page ten.', progress: 'Read both texts\nRewrote the email\nShipped it', updatedAt: NOW, ...over,
});

describe('what a thread is told about its demand', () => {
  it('names the demand, says what it is, and where each other thread stands', () => {
    const block = demandBriefBlock(demand, [sibling('w-a'), sibling('w-b', { status: 'open', label: 'Refund page copy', problem: 'One line still says five days.', progress: '' })]);
    expect(block).toContain('## The demand this thread serves: Refunds a customer can follow');
    expect(block).toContain('The emails and the refund page promise the same thing.');
    expect(block).toContain('- w-a, Refund email rewrite (done): The email promised five days and the page ten. Last step: Shipped it');
    expect(block).toContain('- w-b, Refund page copy (open): One line still says five days.');
    expect(block).toMatch(/leave it a note/);
  });

  it('says it is the first thread of the demand when it is', () => {
    const block = demandBriefBlock(demand, []);
    expect(block).toContain('## The demand this thread serves: Refunds a customer can follow');
    expect(block).toMatch(/No other thread serves it yet/);
    expect(block).not.toMatch(/leave it a note/);
  });

  it('is nothing for a thread with no demand', () => {
    expect(demandBriefBlock(null, [sibling('w-a')])).toBe('');
  });

  it('shows the newest eight siblings and says how many it left out', () => {
    const many = Array.from({ length: 11 }, (_, i) => sibling(`w-${i}`, { updatedAt: NOW + i }));
    const block = demandBriefBlock(demand, many);
    expect(block.match(/^- w-/gm)).toHaveLength(SIBLINGS_SHOWN);
    expect(block).toContain('- w-10,');
    expect(block).not.toContain('- w-2,');
    expect(block).toContain('and 3 older ones');
  });

  it('stays a paragraph or two however much the siblings wrote', () => {
    const wordy = Array.from({ length: 8 }, (_, i) => sibling(`w-${i}`, { problem: 'x'.repeat(5000), progress: 'y'.repeat(5000) }));
    expect(demandBriefBlock(demand, wordy).length).toBeLessThan(3200);
  });
});

describe('the app, briefing a thread', () => {
  const dir = mkdtempSync(join(tmpdir(), 'demand-brief-'));
  writeFileSync(join(dir, 'project.json'), JSON.stringify({ schemaVersion: 1, id: 'p', name: 'Shop' }));
  const product = { slug: 'shop', name: 'Shop', dir, repoPath: null };
  const items = [sibling('w-a', { product: 'shop' }), sibling('w-me', { product: 'shop', status: 'open', label: 'Refund page copy' }), sibling('w-loose', { product: 'shop', status: 'open' })];
  const demands = new DemandStore();
  const made = demands.make(dir, { name: 'Refunds a customer can follow' }, NOW);
  demands.place(dir, 'w-a', made.id, { source: 'agent' }, NOW);
  demands.place(dir, 'w-me', made.id, { source: 'agent' }, NOW);
  const sup = new Supervisor(
    { home: '/nonexistent-home-with-no-discovered-accounts', storeRoot: dir, claudeBin: '/nonexistent/claude', claudeFound: true, maxConcurrentSessions: 3, autonomousProducts: ['shop'] },
    {
      listItems: () => items, listProducts: () => [product], getProduct: () => product, isDue: () => true,
      settleAnswer() {}, recordSessionResult() {}, listRepeats: () => [], demands, productDir: () => dir,
      readItem: (_slug: string, id: string) => items.find((i) => i.id === id) ?? null,
    } as never,
    '/nonexistent-app',
  );

  it('tells a thread of a demand about the demand and the other thread, and not about itself', () => {
    const block = sup.namedThreadsBlock(items[1] as never, product as never, []);
    expect(block).toContain('## The demand this thread serves: Refunds a customer can follow');
    expect(block).toContain('- w-a, Refund email rewrite (done)');
    expect(block).not.toContain('- w-me,');
  });

  it('tells a thread with no demand nothing', () => {
    expect(sup.namedThreadsBlock(items[2] as never, product as never, [])).toBe('');
    rmSync(dir, { recursive: true, force: true });
  });
});
