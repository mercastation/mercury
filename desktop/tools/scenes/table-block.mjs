// A block picked: a press, then Shift and a press two rows down and two columns on; the count
// says how many cells.
import { sheet } from './table.mjs';

export const go = (h) => sheet(h, async (hand) => {
  await hand.tap(...(await hand.cell(1, 0)));
  await hand.tap(...(await hand.cell(3, 2)), { shift: true });
});
