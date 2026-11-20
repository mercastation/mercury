// A DEMAND IS NAMED AND REPORTED IN THE LANGUAGE THE PERSON WRITES IN.
//
// What broke, read off one person's store on 2026-11-20, an hour after demands
// shipped: 8 of 35 demands were named in English and 4 had an English report,
// beside the rest in Portuguese. The person writes every ask in Portuguese.
// The prompts asked for "the language the thread is written in", and a thread
// is mostly an agent's notes: some agents had written the Context in English,
// and an ask like "implementa <a link>" is one word against a paragraph.
//
// So the model is shown how the person writes: a few of their own latest asks,
// their project's first, and told to follow those whatever language the notes
// are in. The same goes for the name of a thread asked by its ticket alone,
// which is named from the agent's notes for the same reason.
import { describe, it, expect } from 'vitest';
import { readFileSync } from 'node:fs';
import { howThePersonWrites, namePrompt } from '../main/row-label.mts';
import { placePrompt } from '../main/demand-sorter.mts';
import { reportPrompt } from '../main/demand-report.mts';

const T0 = 1_795_000_000_000;
let n = 0;
const ask = (title: string, over: Record<string, unknown> = {}) => ({
  id: `w-${n += 1}`, product: 'shop', status: 'open', labels: ['founder'], title, body: title, createdAt: T0 + n, ...over,
});
const flat = (s: string) => s.replace(/\s+/g, ' ');
const VOICE = ['precisamos rever a página de preços antes do lançamento', 'o email de reembolso está saindo com a loja errada'];

describe('how the person writes', () => {
  const here = ask('implementa ABC-1528');

  it('is their own latest asks, newest first', () => {
    const items = [ask('precisamos rever a página de preços'), ask('o email de reembolso está saindo errado'), here];
    expect(howThePersonWrites(here, items)).toEqual(['o email de reembolso está saindo errado', 'precisamos rever a página de preços']);
  });

  it('puts the thread\'s own project first, then the others', () => {
    const items = [ask('o email de reembolso está saindo errado'), ask('escreva o post sobre o lançamento', { product: 'blog' }), here];
    expect(howThePersonWrites(here, items)).toEqual(['o email de reembolso está saindo errado', 'escreva o post sobre o lançamento']);
  });

  it('is never what an agent or the app wrote', () => {
    const items = [
      ask('Follow up: the refund email template', { labels: [] }),
      ask('Your Mac is out of free memory', { kind: 'cause' }),
      ask('Meeting — 2:00 PM', { body: 'I recorded a meeting: [Meeting — 2:00 PM](transcripts/2026-01-06-1400-meeting.md) (42 min).\n\nWrite the notes.' }),
      ask('precisamos rever a página de preços'),
      here,
    ];
    expect(howThePersonWrites(here, items)).toEqual(['precisamos rever a página de preços']);
  });

  it('leaves out an ask that is a link or a ticket and too few words to tell a language by', () => {
    const items = [ask('https://tickets.example.com/browse/ABC-1474'), ask('implementa https://tickets.example.com/browse/ABC-1528'), ask('ABC-1304'), ask('vamos precisar fazer esse card? https://tickets.example.com/browse/ABC-770'), here];
    expect(howThePersonWrites(here, items)).toEqual(['vamos precisar fazer esse card?']);
  });

  it('is a few short lines, not their history', () => {
    const many = Array.from({ length: 12 }, (_, i) => ask(`pedido número ${i} sobre a página de preços ${'e mais detalhes '.repeat(20)}`));
    const lines = howThePersonWrites(here, [...many, here]);
    expect(lines.length).toBe(4);
    expect(Math.max(...lines.map((l) => l.length))).toBeLessThanOrEqual(120);
  });

  it('is nothing for someone who has written nothing else', () => {
    expect(howThePersonWrites(here, [here])).toEqual([]);
  });
});

describe('a new demand\'s name', () => {
  const thread = { id: 'w-1', title: 'implementa ABC-1528', problem: 'You asked to implement ABC-1528. Each store and carrier pair gets a handling time.' };

  it('is asked for in the person\'s language, whatever the notes are in', () => {
    const prompt = flat(placePrompt(thread, [], { voice: VOICE }));
    expect(prompt).toMatch(/in the language the person writes in, shown at the end/);
    expect(prompt).toMatch(/whatever language the thread's notes or the other demands are in/);
    expect(prompt).not.toMatch(/in the language the thread is written in/);
  });

  it('is shown how the person writes, last', () => {
    const prompt = placePrompt(thread, [{ name: 'Price list' }], { voice: VOICE });
    expect(prompt.indexOf('--- how the person writes ---')).toBeGreaterThan(prompt.indexOf('--- the demands ---'));
    for (const line of VOICE) expect(prompt).toContain(line);
  });

  it('falls back to the thread\'s language when the person has written nothing else', () => {
    const prompt = flat(placePrompt(thread, []));
    expect(prompt).toMatch(/in the language the thread is written in/);
    expect(prompt).not.toContain('--- how the person writes ---');
  });
});

describe('a demand\'s report', () => {
  const demand = { name: 'Tempo de manuseio por transportadora' };
  const threads = [{ id: 'w-1', status: 'done', label: 'Handling time per carrier (ABC-1528)', problem: 'You asked to implement ABC-1528.', progress: 'Stored the handling time per pair.' }];

  it('is asked for in the person\'s language, whatever the notes are in', () => {
    const prompt = flat(reportPrompt(demand, threads, { voice: VOICE }));
    expect(prompt).toMatch(/in the language the person writes in, shown at the end/);
    expect(prompt).toMatch(/whatever language the threads' notes are in/);
    expect(prompt).not.toMatch(/in the language the threads are written in/);
  });

  it('is shown how the person writes, last', () => {
    const prompt = reportPrompt(demand, threads, { voice: VOICE });
    expect(prompt.indexOf('--- how the person writes ---')).toBeGreaterThan(prompt.indexOf('--- its threads ---'));
    for (const line of VOICE) expect(prompt).toContain(line);
  });

  it('falls back to the threads\' language when the person has written nothing else', () => {
    expect(reportPrompt(demand, threads)).not.toContain('--- how the person writes ---');
  });
});

describe('the name of a thread asked by its ticket alone', () => {
  const thin = { status: 'open', title: 'implementa ABC-1528', body: '', problem: 'You asked to implement ABC-1528. Each store and carrier pair gets a handling time.' };

  it('is asked for in the person\'s language, because it is named from an agent\'s notes', () => {
    const prompt = flat(namePrompt(thin, { voice: VOICE }));
    expect(prompt).toMatch(/in the language the person writes in, shown at the end/);
    expect(prompt).not.toContain('Write it in the language the opening message is written in.');
    for (const line of VOICE) expect(prompt).toContain(line);
  });

  it('is not shown it when the ask says what it wants in the person\'s own words', () => {
    const prompt = namePrompt({ status: 'open', title: 'precisamos rever a página de preços antes do lançamento', body: '' }, { voice: VOICE });
    expect(prompt).not.toContain('--- how the person writes ---');
    expect(prompt).toContain('Write it in the language the opening message is written in.');
  });
});

describe('the app', () => {
  const supervisor = readFileSync(new URL('../main/supervisor.mts', import.meta.url), 'utf8');

  it('shows the model how the person writes for a name, a placing and a report', () => {
    expect(supervisor).toMatch(/nameRow\(next, \{ \.\.\.opts, others, voice \}\)/);
    expect(supervisor).toMatch(/placePrompt\(next, open, \{ voice \}\)/);
    expect(supervisor).toMatch(/reportPrompt\(demand, threads, \{ voice \}\)/);
    expect(supervisor.match(/howThePersonWrites\(/g)?.length).toBe(3);
  });
});
