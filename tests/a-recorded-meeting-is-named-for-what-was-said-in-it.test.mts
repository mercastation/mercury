// A RECORDED MEETING IS NAMED FOR WHAT WAS SAID IN IT.
//
// What broke, read off one person's store on 2026-11-20: threads named
// "Meeting — 6:17 pm" and "Meeting — 6:25 PM", which say when and nothing of
// what. A recording's thread opens with "I recorded a meeting: [Meeting — 6:17
// PM](transcripts/…) (40 min)." and instructions, in English whatever language
// the meeting was in. The namer runs once, when the thread is made, and the
// only subject in front of it was the hour. What the meeting was about arrives
// when the agent has read the transcript and written its Context.
//
// So a recording's thread waits for the Context or the answer before it is
// named, longer than a ticket's does because a transcript takes longer to
// read, and the prompt is told to name what was discussed, in the language the
// notes are in, never the time. Until then the row shows its own title.
import { describe, it, expect } from 'vitest';
import { readFileSync } from 'node:fs';
import { askedByRecordingAlone, namePrompt, wantsName, waitsForItsSubject, RECORDING_WAIT_MS, THIN_ASK_WAIT_MS } from '../main/row-label.mts';

const T0 = 1_795_000_000_000;
const NOTES = 'Write the notes of this meeting. File each task that came out of it for me as its own proposal under this thread.';
const recording = (over: Record<string, unknown> = {}) => ({
  status: 'open', title: 'Meeting — 2:00 PM', createdAt: T0,
  body: `I recorded a meeting: [Meeting — 2:00 PM](transcripts/2026-01-06-1400-meeting-2-00-pm.md) (42 min).\n\n${NOTES}`,
  ...over,
});
const flat = (s: string) => s.replace(/\s+/g, ' ');

describe('an ask that is a recording and nothing of what was said', () => {
  it('is one, from the Mac', () => {
    expect(askedByRecordingAlone(recording())).toBe(true);
  });

  it('is one, from the phone', () => {
    expect(askedByRecordingAlone(recording({ body: `I recorded a meeting on my phone: [Meeting 8 Jan, 17:57](transcripts/2026-01-08-1757-meeting.md) (3 min). The audio is a.m4a.\n\n${NOTES}` }))).toBe(true);
  });

  it('is not one when a person only writes about a meeting', () => {
    expect(askedByRecordingAlone({ status: 'open', title: 'Supplier follow-up', body: 'I recorded a meeting with the supplier: please write the follow-up email.' })).toBe(false);
    expect(askedByRecordingAlone({ status: 'open', title: 'Notes from when I recorded a meeting with the supplier.', body: '' })).toBe(false);
  });
});

describe('when a recording\'s thread is named', () => {
  it('waits while the agent has not said what the meeting was about', () => {
    expect(waitsForItsSubject(recording(), T0 + 60_000)).toBe(true);
    expect(waitsForItsSubject(recording(), T0 + RECORDING_WAIT_MS - 1)).toBe(true);
  });

  it('waits longer than a thread asked by its ticket, because a transcript takes longer to read', () => {
    expect(RECORDING_WAIT_MS).toBeGreaterThan(THIN_ASK_WAIT_MS);
  });

  it('waits through a note that only says the agent has started', () => {
    expect(waitsForItsSubject(recording({ note: 'Reading the transcript.' }), T0 + 60_000)).toBe(true);
  });

  it('is named as soon as the Context or the answer is written', () => {
    expect(waitsForItsSubject(recording({ problem: 'Notes of the pricing review: the yearly plan and the two discounts.' }), T0 + 60_000)).toBe(false);
    expect(waitsForItsSubject(recording({ result: 'The meeting settled the yearly plan. Three tasks came out of it.' }), T0 + 60_000)).toBe(false);
  });

  it('is named anyway once the wait is over, so nothing waits for ever', () => {
    expect(waitsForItsSubject(recording(), T0 + RECORDING_WAIT_MS)).toBe(false);
  });

  it('is still wanted by the namer all along', () => {
    expect(wantsName(recording())).toBe(true);
  });
});

describe('the naming prompt', () => {
  it('asks for what the meeting was about, in the notes\' language, and never its time', () => {
    const prompt = flat(namePrompt(recording({ problem: 'Notas da revisão de preços: o plano anual e os dois descontos.' })));
    expect(prompt).toMatch(/only says a meeting was recorded/);
    expect(prompt).toMatch(/name what the meeting was about/);
    expect(prompt).toMatch(/never by its time/);
    expect(prompt).toMatch(/in the language that is written in, not the opening line's/);
    expect(prompt).toContain('Notas da revisão de preços');
  });

  it('does not also ask for the opening line\'s language, which is the app\'s own English', () => {
    expect(namePrompt(recording())).not.toContain('Write it in the language the opening message is written in.');
  });

  it('says none of that on a thread a person opened in their own words', () => {
    const prompt = flat(namePrompt({ status: 'open', title: 'Rever a página de preços', body: '', createdAt: T0 }));
    expect(prompt).not.toMatch(/meeting was recorded/);
    expect(prompt).toContain('Write it in the language the opening message is written in.');
  });
});

describe('a recording', () => {
  it('is told apart the same way by the namer and by the sorter that keeps it out of demands', () => {
    expect(readFileSync(new URL('../main/demand-sorter.mts', import.meta.url), 'utf8')).toContain('askedByRecordingAlone(item)');
  });
});
