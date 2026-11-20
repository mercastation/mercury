// A THREAD IS NAMED IN THE LANGUAGE IT WAS ASKED IN.
//
// What broke, read off one person's store on 2026-11-20: they write to their
// agents in Portuguese, and about 24 of the threads opened that way were named
// in English, beside others named in Portuguese. One list in two languages
// reads as two lists. The naming prompt said nothing about language, so the
// model answered in the language of its own instructions.
import { describe, it, expect } from 'vitest';
import { namePrompt } from '../main/row-label.mts';

const flat = (s: string) => s.replace(/\s+/g, ' ');

describe('the naming prompt', () => {
  it('asks for the name in the language of the opening message', () => {
    expect(flat(namePrompt({ title: 'o relatório semanal está ficando longo demais', body: '' })))
      .toMatch(/in the language the opening message is written in/);
  });

  it('hands the opening message over as it was written, accents and all', () => {
    const prompt = namePrompt({ title: 'a página de preços não mostra o cupão', body: 'Só acontece no telemóvel.' });
    expect(prompt).toContain('a página de preços não mostra o cupão');
    expect(prompt).toContain('Só acontece no telemóvel.');
  });

  it('keeps the rules it already had', () => {
    const prompt = flat(namePrompt({ title: 'add a /health endpoint', body: '' }));
    expect(prompt).toMatch(/3 to 6 words/);
    expect(prompt).toMatch(/in the words the opening line uses/);
  });
});
