// A field of a row opened whole written: the company's box pressed past its words, words typed
// on, and ⌘Enter giving the keys back to the panel, which writes the field (`onBlur`). The
// fixture keeps what the page would have written (`wrote`), for the native window's words to be
// held to.
import { sheet } from './table.mjs';

export const go = (h) => sheet(h, async (hand) => {
  await hand.tap(...(await hand.cell(1, 0)));
  await hand.key(' ');
  await hand.wait(300);
  // Past the end of its words, so the caret lands after them in both windows.
  const end = await h.evaluate(`(() => {
    const t = document.querySelector('.tbl-sheet .tbl-field[data-field="0"] textarea');
    if (!t) return null;
    const r = t.getBoundingClientRect();
    return [Math.round(r.right - 6), Math.round(r.top + r.height / 2)];
  })()`);
  if (!end) throw new Error('table-field: no company box');
  await hand.tap(...end);
  await hand.type(' Corporation');
  await hand.key('Enter', { cmd: true });
});
