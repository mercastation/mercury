// The code a run changed, invented in the engine's own shape (`SavedChange` in
// main/code-change.mts): files in folders, each a list of hunks whose rows are `['=' | '+' | '-',
// line]` with the line numbers git gave them (`nums`, `[old, new]`), the run's start and end,
// and where its folder stood (`committed`, `uncommitted`).
//
// The harness's bridge answers no change, so a scene answers `zero:code-change` in the page
// (`answersTheChange`): the change without its rows when the page asks for a summary, as the
// strip's button and the steps' chips do, and the whole of it when the pane opens it. The files
// it changed are planted in the thread's code folder as they stand after the run, so a gap
// between two hunks opens onto the same lines in both windows (`zero:code-file` is answered from
// them too). Each scene hands the same answers on in the fixture: the summary under the
// thread's `zero:code-change`, the whole under `codeChanges`.
import fs from 'node:fs';
import path from 'node:path';

const MIN = 60_000;

/** What the files say after the run: what is planted, and what the gaps open onto. */
export const AFTER = {
  'src/checkout/tax.ts': `// Tax on a cart, line by line, as the receipt shows it.
import { cents } from '../money';
import type { CartLine } from './cart';

export type TaxLine = { label: string; amount: number };

/** Every line rounded to the cent, then added: the receipt matches the charge. */
export function roundTax(amount: number): number {
  return Math.round(amount * 100) / 100;
}

export function taxLines(lines: CartLine[], rate: number): TaxLine[] {
  return lines.map((line) => ({
    label: line.label,
    amount: roundTax(line.price * line.quantity * rate),
  }));
}

export function taxRate(region: string): number {
  switch (region) {
    case 'north': return 0.0825;
    case 'south': return 0.07;
    case 'east': return 0.0625;
    default: return 0.06;
  }
}
`,
  'src/checkout/total.ts': `// What a cart comes to, tax included.
import { cents } from '../money';
import type { CartLine } from './cart';
import { taxLines, taxRate, totalTax } from './tax';

export function cartTotal(lines: CartLine[], region: string): number {
  const goods = lines.reduce((sum, line) => sum + line.price * line.quantity, 0);
  const tax = totalTax(taxLines(lines, taxRate(region)));
  return cents(goods + tax);
}
`,
  'src/checkout/tax.test.ts': `import { describe, expect, it } from 'vitest';
import { roundTax, totalTax } from './tax';

describe('the tax on a cart', () => {
  it('rounds a line to the cent', () => {
    expect(roundTax(1240.555)).toBe(1240.56);
  });

  it('adds the lines after rounding each one, so the receipt a customer sees always matches what the card is charged', () => {
    const lines = [{ label: 'Lamp', amount: 87.115 }, { label: 'Shade', amount: 12.004 }];
    expect(totalTax(lines)).toBe(99.12);
  });
});
`,
  'src/money.ts': `// Money is kept in dollars and rounded to the cent at the edges.
export const cents = (n: number): number => Math.round(n * 100) / 100;
`,
  'notes/refunds-for-support.md': `# Refunds for the 37 orders

Each order over $1,000 since 08:10 is refunded the cent it was over-charged.
The refunds wait until support has read this note.
Nothing goes out to a customer before then.
`,
};

/** A hunk from rows written as `[mark, old number, new number, line]`. */
const hunk = (rows) => {
  const out = { rows: rows.map(([m, , , t]) => [m, t]), nums: rows.map(([, o, n]) => [o, n]) };
  out.plus = out.rows.filter((r) => r[0] === '+').length;
  out.minus = out.rows.filter((r) => r[0] === '-').length;
  return out;
};
/** A file every line of which is new, or gone. */
const whole = (text, mark) => {
  const lines = text.replace(/\n$/, '').split('\n');
  return hunk(lines.map((t, i) => (mark === '+' ? ['+', null, i + 1, t] : ['-', i + 1, null, t])));
};
const lines = (rel) => AFTER[rel].replace(/\n$/, '').split('\n');

/** The change, its files under the code folder `repo`, its run six minutes long ending at `at`. */
export function theChange(repo, at, { item = 'w-readme0', committed = false } = {}) {
  const tax = lines('src/checkout/tax.ts');
  const ctx = (n) => ['=', n, n, tax[n - 1]];
  const files = [
    {
      path: 'src/checkout/tax.ts',
      hunks: [
        hunk([ctx(4), ctx(5), ctx(6),
          ['-', 7, null, '/** Rounds a tax amount to two places. */'], ['+', null, 7, tax[6]],
          ctx(8),
          ['-', 9, null, '  return Number(amount.toFixed(2));'], ['+', null, 9, tax[8]],
          ctx(10), ctx(11), ctx(12)]),
        hunk([ctx(18), ctx(19), ctx(20),
          ['-', 21, null, "    case 'north': return 0.08;"], ['+', null, 21, tax[20]],
          ctx(22),
          ['-', 23, null, "    case 'east': return 0.06;"], ['+', null, 23, tax[22]],
          ctx(24), ctx(25), ctx(26)]),
      ],
    },
    {
      path: 'src/checkout/total.ts',
      hunks: [hunk([
        ['=', 5, 5, ''],
        ['-', 6, null, 'export function cartTotal(lines: CartLine[]): number {'],
        ['+', null, 6, lines('src/checkout/total.ts')[5]],
        ['=', 7, 7, lines('src/checkout/total.ts')[6]],
        ['-', 8, null, '  const tax = totalTax(lines.map(toTaxLine));'],
        ['+', null, 8, lines('src/checkout/total.ts')[7]],
        ['=', 9, 9, lines('src/checkout/total.ts')[8]],
        ['=', 10, 10, '}'],
      ])],
    },
    { path: 'src/checkout/tax.test.ts', hunks: [whole(AFTER['src/checkout/tax.test.ts'], '+')] },
    {
      path: 'src/money.ts',
      hunks: [hunk([
        ['=', 1, 1, lines('src/money.ts')[0]],
        ['-', 2, null, 'export const cents = (n: number) => Math.round(n * 100) / 100;'],
        ['+', null, 2, lines('src/money.ts')[1]],
      ])],
    },
    { path: 'notes/refunds-for-support.md', hunks: [whole(AFTER['notes/refunds-for-support.md'], '+')] },
    {
      path: 'scripts/retry-declined.mjs',
      hunks: [whole(`// Retries a declined card once, after a second.
import { charge } from '../src/checkout/pay.mjs';

export async function retryDeclined(order) {
  await new Promise((done) => setTimeout(done, 1000));
  return charge(order);
}
`, '-')],
    },
  ].map((f) => ({ ...f, abs: path.join(repo, f.path), plus: f.hunks.reduce((n, h) => n + h.plus, 0), minus: f.hunks.reduce((n, h) => n + h.minus, 0) }));
  return {
    item,
    files,
    roots: [repo],
    plus: files.reduce((n, f) => n + f.plus, 0),
    minus: files.reduce((n, f) => n + f.minus, 0),
    editCount: 9,
    withProse: 9,
    startedAt: at - 6 * MIN,
    endedAt: at,
    committed,
    uncommitted: true,
    knowsYou: true,
    fromDisk: files.length,
  };
}

/** The change without its rows, as `summary: true` answers it (`readChangeSummary`). */
export const summaryOf = (change) => ({ ...change, files: change.files.map(({ hunks: _rows, ...f }) => f) });

/** Where a thread's change is (`changePathFor` in renderer/src/code-artifact.ts). */
export const changePathFor = (item) => `runs/${item}/the-change-it-made.change`;

/** A thread id as the store makes them, `w-` and hex: the pane names a change at such a path
 * "The change" (`DocPane.tsx`), and the invented inbox's ids are not hex. For `open`'s `dress`. */
export const CHANGED_THREAD = 'w-c0de5a1e01';
export const hexId = (row) => { row.id = CHANGED_THREAD; };

/**
 * The thread's change, for `open` in ./thread.mjs (its `change`): plants the files as they stand
 * after the run in the planted code folder and gives what the page's reads are answered with:
 * the summary, the whole change, its path and the files a gap or a save reads. `pending` leaves
 * the whole change unanswered, for the skeleton; `refused` answers it with the engine's refusal;
 * `make` builds another change on the same folder (a large one, say).
 */
export function changeFor({ pending = false, refused = null, committed = false, make = theChange } = {}) {
  return (planted, row) => {
    const repo = planted.repo;
    for (const [rel, text] of Object.entries(AFTER)) {
      const file = path.join(repo, rel);
      fs.mkdirSync(path.dirname(file), { recursive: true });
      fs.writeFileSync(file, text);
    }
    const change = make(repo, row.wrote?.result?.ts ?? row.updatedAt, { item: row.id, committed });
    const files = Object.fromEntries(change.files.filter((f) => fs.existsSync(f.abs)).map((f) => [f.path, { ok: true, path: f.abs, text: fs.readFileSync(f.abs, 'utf8'), mtime: fs.statSync(f.abs).mtimeMs }]));
    // The change file itself, where the supervisor writes it, which the pane's finder finds.
    const file = path.join(planted.dir, changePathFor(row.id));
    fs.mkdirSync(path.dirname(file), { recursive: true });
    fs.writeFileSync(file, JSON.stringify(change));
    return {
      file,
      src: changePathFor(row.id),
      summary: { ok: true, change: summaryOf(change) },
      whole: refused ? { ok: false, error: refused } : { ok: true, change },
      files,
      pending,
    };
  };
}

/** Presses the strip's code button, as a hand does. */
export async function pressTheButton(h) {
  const at = await h.evaluate(`(() => { const b = document.querySelector('.ts-code'); if (!b) return null; const r = b.getBoundingClientRect(); return [r.x + r.width / 2, r.y + r.height / 2]; })()`);
  if (!at) throw new Error('change scene: no code button in the strip');
  await h.call('Input.dispatchMouseEvent', { type: 'mouseMoved', x: at[0], y: at[1] });
  await h.call('Input.dispatchMouseEvent', { type: 'mousePressed', x: at[0], y: at[1], button: 'left', clickCount: 1 });
  await h.call('Input.dispatchMouseEvent', { type: 'mouseReleased', x: at[0], y: at[1], button: 'left', clickCount: 1 });
  // And the pointer off it, so the picture is of the button at rest.
  await h.call('Input.dispatchMouseEvent', { type: 'mouseMoved', x: 10, y: 400 });
  await h.wait(900);
}
