// Cells typed over and written: a name, an amount typed plain (written as the column writes
// them), a day typed in words, a tag picked from its list with the keys, and a row put in with
// Shift Enter. The fixture keeps what the page would have written (`wrote`), for the native
// window's words to be held to.
import { sheet } from './table.mjs';

export const go = (h) => sheet(h, async (hand) => {
  await hand.tap(...(await hand.cell(2, 0)));
  await hand.key('Enter');
  await hand.type(' Corporation');
  await hand.key('Enter');
  await hand.tap(...(await hand.cell(4, 1)));
  await hand.key('ArrowDown');
  await hand.key('Enter');
  await hand.tap(...(await hand.cell(4, 0)));
  await hand.key('Enter', { shift: true });
  await hand.type('Wayne Fabrics');
  await hand.key('Tab');
  await hand.key('Escape');
  await hand.key('Escape');
});
