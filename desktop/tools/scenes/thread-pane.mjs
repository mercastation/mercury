// A file opened in the pane beside the conversation (`DocPane` in
// renderer/src/components/DocPane.tsx): the conversation narrowed on the left, the file's card on
// the right with its path, its marks, when it was last edited and the file drawn.
//
// The pane reads the file through the app (`zero:open-artifact` asked to resolve, then
// `zero:read-doc`), neither of which the harness answers, so once the thread is open both are
// answered in the page from the planted folder, with the file's own time on disk as the native
// window reads it. The pane draws its globe only where the app says files can be shared
// (`zero:file-link` naming a site), which is answered as a signed-in window with nothing shared
// yet, and handed on in the fixture (`fileLink`).
//
// `open` here takes `pressed`, the planted path whose mention the scene presses, `split`, the
// fraction a drag leaves the pane at (dragged by its grip, as a hand drags it), and `then`, what
// the scene does once the pane is open (a pointer resting on a mark). Each scene's fixture says
// which file was pressed (`docOpen`) and the split (`docSplit`).
import fs from 'node:fs';
import path from 'node:path';
import { open as openThread } from './thread.mjs';
import { findsFiles } from './thread-file-picture.mjs';

const MIN = 60_000;

/** An invented plan, long enough to scroll, in the shape an agent's plan takes. */
export const PLAN = `# Plan: the receipt, line by line

**In one sentence:** round each tax line to the cent before the lines are added up, so the receipt a customer sees always matches what the card is charged; the old total stays readable for a week; nothing about the cart itself changes.

Second round, after the review: the refunds for the 37 orders are written down below and support has the note. What is left is the order the two changes ship in, and one question about gift cards.

## What changes and what stays

| Part | Today | After |
| --- | --- | --- |
| **The tax line** | Rounded once, at the end | Rounded per line, to the cent |
| **The total** | Added from the raw lines | Added from the rounded lines |
| **The receipt** | Can be a cent off the charge | Always matches the charge |
| **The cart** | Unchanged | Unchanged |

## How it ships

1. The rounding fix, on its own, behind no switch.
2. The 37 refunds, once support has read the note.
3. The old total kept beside the new one for a week, then taken away.

Each step is small enough to take back in a minute. The fix is one function and its tests; the refunds are a queue that waits on the note; the old total is a second line on the receipt that nobody reads after the first week.

## What is left

Two things are not decided yet: whether a refund shows on the same receipt as the order it refunds, and how a gift card is named on a receipt that has both. Neither holds up the fix.

## Numbers

The 37 failed orders came to $48,211.40 in all. The largest was $5,902.10 and the smallest $1,004.99. Every one of them was over $1,000, and none was paid with store credit.

> The tax service still answers with four places. If it ever goes back to two, nothing here needs to change.
`;

/** A script, for the pane's text view. */
export const SCRIPT = `// Rounds each tax line to the cent before the lines are added up.
import { cents } from './money';

export type TaxLine = { label: string; amount: number };

/** Every line rounded, then added: the receipt always matches the charge. */
export function totalTax(lines: TaxLine[]): number {
  let total = 0;
  for (const line of lines) {
    total += roundTax(line.amount);
  }
  return cents(total);
}

export function roundTax(amount: number): number {
  return Math.round(amount * 100) / 100;
}
`;

/** A sheet, for the pane's table. */
export const SHEET = `order,total,tax,failed,refund
1001,1049.99,87.12,yes,8.40
1002,412.00,34.18,no,
1003,5902.10,489.87,yes,47.22
1004,1004.99,83.41,yes,8.04
1005,2210.55,183.47,yes,17.68
1006,880.00,73.04,no,
1007,1518.30,126.02,yes,12.15
`;

/** The extra files these scenes name, written into the planted folder once it is planted. */
const EXTRA = { 'designs/receipt-plan.md': PLAN, 'src/round-tax.ts': SCRIPT, 'exports/orders.csv': SHEET };

/** A signed-in window with nothing shared yet, on a cloud that shares files. */
export const FILE_LINK = { ok: true, site: 'https://files.example.com', signedIn: true, link: null };

/** Writes the extra files, edited three minutes before `now`, and answers the pane's reads from them. */
async function answersThePane(h, now) {
  const dir = await h.evaluate(`(window.__SNAP__.products.find((p) => p.dir && p.dir.includes('/storefront')) ?? {}).dir ?? ''`);
  if (!dir) throw new Error('pane scene: no planted project');
  const texts = {};
  for (const [rel, words] of Object.entries(EXTRA)) {
    const file = path.join(dir, rel);
    fs.mkdirSync(path.dirname(file), { recursive: true });
    fs.writeFileSync(file, words);
    const when = new Date(now - 3 * MIN);
    fs.utimesSync(file, when, when);
    texts[file] = { text: words, mtime: fs.statSync(file).mtimeMs };
  }
  for (const rel of ['notes/rounding.md', 'notes/rounding-v2.md', 'exports/failed-orders.csv', 'exports/refunds.csv']) {
    const file = path.join(dir, rel);
    if (fs.existsSync(file)) texts[file] = { text: fs.readFileSync(file, 'utf8'), mtime: fs.statSync(file).mtimeMs };
  }
  await findsFiles(h);
  await h.evaluate(`(() => {
    const texts = ${JSON.stringify(texts)};
    const dirOf = (product) => (window.__SNAP__.products.find((p) => p.slug === product) ?? {}).dir ?? '';
    const whole = (product, src) => { const p = String(src ?? '').replace(/^file:\\/\\//, '').split(/[?#]/)[0].replace(/^\\.\\//, ''); return p.startsWith('/') ? p : dirOf(product) + '/' + p; };
    window.zero.readDoc = async ({ product, src } = {}) => { const t = texts[whole(product, src)]; return t ? { ok: true, text: t.text, mtime: t.mtime } : { ok: false, error: 'That file could not be found.' }; };
    window.zero.fileLink = async () => (${JSON.stringify(FILE_LINK)});
  })()`);
}

/** Presses the mention of `pressed` in the conversation, as a hand would. */
export async function pressMention(h, pressed) {
  const hit = await h.evaluate(`(() => {
    const a = [...document.querySelectorAll('.focus-body a')].find((a) => (a.getAttribute('href') || '').endsWith(${JSON.stringify(pressed)}));
    if (!a) return null;
    const r = a.getBoundingClientRect();
    return [r.x + r.width / 2, r.y + r.height / 2];
  })()`);
  if (!hit) throw new Error(`pane scene: no mention of ${pressed}`);
  await click(h, hit[0], hit[1]);
  await h.wait(1500);
}

export async function click(h, x, y) {
  await h.call('Input.dispatchMouseEvent', { type: 'mouseMoved', x, y });
  await h.call('Input.dispatchMouseEvent', { type: 'mousePressed', x, y, button: 'left', clickCount: 1 });
  await h.call('Input.dispatchMouseEvent', { type: 'mouseReleased', x, y, button: 'left', clickCount: 1 });
}

/** Drags the pane's grip, as a hand does, to leave the pane at `split` of the body. */
export async function dragTo(h, split) {
  const at = await h.evaluate(`(() => {
    const g = document.querySelector('.doc-grip'); const body = g?.closest('.body');
    if (!g || !body) return null;
    const r = g.getBoundingClientRect(); const b = body.getBoundingClientRect();
    return { x: r.x + r.width / 2, y: r.y + r.height / 2, left: b.left, width: b.width };
  })()`);
  if (!at) throw new Error('pane scene: no grip');
  // splitFromPointer reads the pointer against the body's left and the card's own inset.
  const inset = await h.evaluate(`document.querySelector('.doc-card')?.offsetLeft ?? 0`);
  const to = at.left + inset + at.width * (1 - split);
  await h.call('Input.dispatchMouseEvent', { type: 'mouseMoved', x: at.x, y: at.y });
  await h.call('Input.dispatchMouseEvent', { type: 'mousePressed', x: at.x, y: at.y, button: 'left', clickCount: 1 });
  for (let k = 1; k <= 8; k += 1) {
    await h.call('Input.dispatchMouseEvent', { type: 'mouseMoved', x: at.x + ((to - at.x) * k) / 8, y: at.y, button: 'left', buttons: 1 });
  }
  await h.call('Input.dispatchMouseEvent', { type: 'mouseReleased', x: to, y: at.y, button: 'left', clickCount: 1 });
  // And the pointer away from the grip, whose pill stays only while it is under the pointer.
  await h.call('Input.dispatchMouseEvent', { type: 'mouseMoved', x: 10, y: 400 });
  await h.wait(400);
  return h.evaluate(`Number(localStorage.getItem('zero.docPane.split.v1'))`);
}

const SHIPPED = `**The plan is written down.**

It is in designs/receipt-plan.md, with the refunds at its foot. The rounding itself is in \`src/round-tax.ts\` and the orders it touched are in exports/orders.csv.`;

export function open(h, { pressed = 'designs/receipt-plan.md', split = null, shipped = SHIPPED, then = null } = {}) {
  return openThread(h, {
    unfold: false,
    shipped,
    files: {},
    then: async (h) => {
      const now = await h.evaluate('Date.now()');
      await answersThePane(h, now);
      if (pressed) await pressMention(h, pressed);
      const docSplit = split === null ? null : await dragTo(h, split);
      const more = then ? (await then(h)) ?? {} : {};
      return { ...(pressed ? { docOpen: pressed } : {}), fileLink: FILE_LINK, ...(docSplit ? { docSplit } : {}), ...more };
    },
  });
}

export const go = (h) => open(h);
