// A THREAD ASKED BY ITS TICKET IS NAMED FOR WHAT THE TICKET IS.
//
// What broke, read off one person's store on 2026-11-20: 13 of 97 threads were
// named by a ticket number and nothing else ("Implement ABC-1496", "ABC-1304
// implementation"), so a list of them said nothing about the work. Each was
// opened with a line like "implement ABC-1496", and the namer runs once, when
// the thread is made, with only that line to read. What the ticket was about
// arrived minutes later, in the agent's first Context, after the name was set.
//
// Two changes. A thread asked by its ticket alone waits for the agent's first
// words before it is named, ten minutes at most. And the prompt reads the
// Context, and is told to name the subject and put the id last in brackets,
// so the id the person would look for is still on the row.
import { describe, it, expect } from 'vitest';
import { readFileSync } from 'node:fs';
import { askedByTicketAlone, cleanName, namePrompt, wantsName, waitsForItsSubject, THIN_ASK_WAIT_MS } from '../main/row-label.mts';

const T0 = 1_795_000_000_000;
const asked = (title: string, over: Record<string, unknown> = {}) => ({ status: 'open', title, body: '', createdAt: T0, ...over });
const flat = (s: string) => s.replace(/\s+/g, ' ');

describe('an ask that is a ticket and little else', () => {
  it.each(['implement ABC-1496', 'ABC-1304 implementation', 'Status of ticket ABC-1345', 'cards ABC-1303 and ABC-1278'])('is one: "%s"', (title) => {
    expect(askedByTicketAlone(asked(title))).toBe(true);
  });

  it('is not one when the ask says what the work is', () => {
    expect(askedByTicketAlone(asked('implement ABC-1496: each store sets its own handling time for a carrier'))).toBe(false);
    expect(askedByTicketAlone(asked('ABC-1496', { body: 'Each store and carrier pair gets a handling time of its own, used before the store default.' }))).toBe(false);
  });

  it('is not one without a ticket id, however short', () => {
    expect(askedByTicketAlone(asked('add a /health endpoint'))).toBe(false);
    expect(askedByTicketAlone(asked('clean up the pre-2020 rows'))).toBe(false);
    expect(askedByTicketAlone(asked('Ship'))).toBe(false);
  });
});

describe('when such a thread is named', () => {
  const thin = asked('implement ABC-1496');

  it('waits while the agent has written nothing', () => {
    expect(waitsForItsSubject(thin, T0 + 60_000)).toBe(true);
    expect(waitsForItsSubject(thin, T0 + THIN_ASK_WAIT_MS - 1)).toBe(true);
  });

  it('is named as soon as the agent says what it is about', () => {
    expect(waitsForItsSubject({ ...thin, problem: 'Each store and carrier pair gets a handling time of its own.' }, T0 + 60_000)).toBe(false);
    expect(waitsForItsSubject({ ...thin, result: 'Done: the handling time is stored per pair.' }, T0 + 60_000)).toBe(false);
    expect(waitsForItsSubject({ ...thin, note: 'Read the ticket; three parts.' }, T0 + 60_000)).toBe(false);
  });

  it('is named anyway once ten minutes have passed, so nothing waits for ever', () => {
    expect(waitsForItsSubject(thin, T0 + THIN_ASK_WAIT_MS)).toBe(false);
  });

  it('does not hold back a thread that said what it wants', () => {
    expect(waitsForItsSubject(asked('add a /health endpoint'), T0 + 1)).toBe(false);
    expect(waitsForItsSubject(asked('implement ABC-1496: each store sets its own handling time for a carrier'), T0 + 1)).toBe(false);
  });

  it('is still wanted by the namer all along, and never named twice', () => {
    expect(wantsName(thin)).toBe(true);
    expect(wantsName({ ...thin, label: 'Handling time per carrier (ABC-1496)', problem: 'x' })).toBe(false);
  });

  it('is what the app asks before it names a row', () => {
    expect(readFileSync(new URL('../main/supervisor.mts', import.meta.url), 'utf8')).toContain('!waitsForItsSubject(i, now)');
  });
});

describe('the naming prompt', () => {
  it('asks for the subject, with the id last in brackets', () => {
    const prompt = flat(namePrompt(asked('implement ABC-1496')));
    expect(prompt).toMatch(/only by a ticket or card id/);
    expect(prompt).toMatch(/put the id last in brackets/);
  });

  it('reads the Context the agent wrote', () => {
    const prompt = namePrompt(asked('implement ABC-1496', { problem: 'Each store and carrier pair gets a handling time of its own.' }));
    expect(prompt).toContain('--- what it is about ---');
    expect(prompt).toContain('Each store and carrier pair gets a handling time of its own.');
  });

  it('draws no empty Context heading on a thread nobody has summed up', () => {
    expect(namePrompt(asked('implement ABC-1496'))).not.toContain('--- what it is about ---');
  });
});

describe('a name that ends in its ticket', () => {
  it('is kept: the id in brackets is not one of the words counted', () => {
    expect(cleanName('Handling time per store, carrier and delivery mode (ABC-1496)')).toBe('Handling time per store, carrier and delivery mode (ABC-1496)');
  });

  it('is still refused when it runs on', () => {
    expect(cleanName('The handling time that every store sets for each of its carriers (ABC-1496)')).toBe('');
    expect(cleanName('one two three four five six seven eight nine')).toBe('');
  });
});
