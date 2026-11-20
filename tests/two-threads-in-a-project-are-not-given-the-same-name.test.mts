// TWO THREADS IN A PROJECT ARE NOT GIVEN THE SAME NAME.
//
// What broke, read off one person's store on 2026-11-20: six threads in one
// project had the same name, word for word. The same routine job is asked for
// every few days in nearly the same words, and the namer saw one thread at a
// time, so the same words gave the same name and the list could not tell last
// week's run from today's.
//
// The namer is now handed the names its project already uses and told to say
// what is different about this thread. Covered here: the prompt with names and
// without, and which names the app hands over (the same project's, not the
// thread's own, not another project's, not a row nobody has named).
import { describe, it, expect, vi, beforeEach, afterAll } from 'vitest';
import { mkdtempSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { namePrompt, namesTaken } from '../main/row-label.mts';

const fake = vi.hoisted(() => ({ spawns: [] as { args: string[] }[], reply: 'Sync of 14 November\n' }));
vi.mock('node:child_process', async (importActual) => {
  const actual = await importActual<typeof import('node:child_process')>();
  const { EventEmitter: Emitter } = await import('node:events');
  return {
    ...actual,
    spawn: (_bin: string, args: string[]) => {
      fake.spawns.push({ args });
      const child = new Emitter() as InstanceType<typeof Emitter> & { stdout: InstanceType<typeof Emitter>; stderr: InstanceType<typeof Emitter>; kill: () => boolean };
      child.stdout = new Emitter();
      child.stderr = new Emitter();
      child.kill = () => true;
      setTimeout(() => { child.stdout.emit('data', Buffer.from(fake.reply)); child.emit('close', 0); }, 0);
      return child;
    },
  };
});

const { Supervisor } = await import('../main/supervisor.mts');

const flat = (s: string) => s.replace(/\s+/g, ' ');
const row = (id: string, over: Record<string, unknown> = {}) => ({
  id, product: 'shop', kind: 'directive', status: 'open', title: 'sync the fork with upstream again please', body: '',
  labels: [], priority: 5, createdAt: 1, updatedAt: 10, claim: null, ...over,
});

describe('the naming prompt', () => {
  it('lists the names already taken and asks for what is different', () => {
    const prompt = namePrompt(row('w-new'), { others: ['Fork sync', 'Fork sync and publish'] });
    expect(prompt).toContain('--- names already taken ---');
    expect(prompt).toContain('Fork sync\nFork sync and publish');
    expect(flat(prompt)).toMatch(/Never give this one a name any of them has/);
  });

  it('says nothing about other names on a project with none', () => {
    const prompt = namePrompt(row('w-new'));
    expect(prompt).not.toContain('--- names already taken ---');
    expect(flat(prompt)).not.toMatch(/Never give this one a name/);
  });
});

describe('the names a thread may not repeat', () => {
  const rows = [
    row('w-new', { updatedAt: 50 }),
    row('w-a', { label: 'Fork sync' }),
    row('w-b', { label: 'Fork sync and publish', status: 'done' }),
    row('w-c', { label: 'Pricing page', product: 'studio' }),
    row('w-d', { title: 'a long dictated first sentence nobody has named yet' }),
    row('w-e', { label: 'Fork sync' }),
  ];

  it('are the named threads of its own project, each once', () => {
    expect(namesTaken(rows[0], rows)).toEqual(['Fork sync', 'Fork sync and publish']);
  });

  it('never include its own name, another project or a row with no name', () => {
    expect(namesTaken(rows[2], rows)).toEqual(['Fork sync']);
    expect(namesTaken(rows[3], rows)).toEqual([]);
  });

  it('stop at forty, newest first, so the prompt stays small', () => {
    const many = Array.from({ length: 60 }, (_, i) => row(`w-${i}`, { label: `Name ${i}`, updatedAt: i }));
    const taken = namesTaken(row('w-new'), many);
    expect(taken).toHaveLength(40);
    expect(taken[0]).toBe('Name 59');
  });
});

describe('the app, naming a thread', () => {
  const dirs: string[] = [];
  afterAll(() => { for (const d of dirs) { try { rmSync(d, { recursive: true, force: true }); } catch { /* best effort */ } } });
  beforeEach(() => { fake.spawns.length = 0; });

  it('hands the namer the names its project already uses', async () => {
    const dir = mkdtempSync(join(tmpdir(), 'name-twice-'));
    dirs.push(dir);
    const product = { slug: 'shop', name: 'Shop', dir, repoPath: null };
    const named: { id: string; label: string }[] = [];
    const items = [row('w-new', { updatedAt: 50 }), row('w-a', { label: 'Fork sync' }), row('w-c', { label: 'Pricing page', product: 'studio' })];
    const sup = new Supervisor(
      { home: '/nonexistent-home-with-no-discovered-accounts', storeRoot: dir, claudeBin: '/nonexistent/claude', claudeFound: true, maxConcurrentSessions: 3, autonomousProducts: ['shop'] },
      {
        listItems: () => items, listProducts: () => [product], getProduct: () => product, isDue: () => true,
        settleAnswer() {}, recordSessionResult() {}, listRepeats: () => [],
        nameItem: (_slug: string, id: string, label: string) => named.push({ id, label }),
      } as never,
      '/nonexistent-app',
    );
    await sup.nameTheRows(items as never);
    expect(named).toEqual([{ id: 'w-new', label: 'Sync of 14 November' }]);
    const sent = fake.spawns[0].args.join('\n');
    expect(sent).toContain('--- names already taken ---');
    expect(sent).toContain('Fork sync');
    expect(sent).not.toContain('Pricing page');
  });
});
