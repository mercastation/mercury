// A row opened whole with Space: every field of it in a panel at the table's right.
import { sheet } from './table.mjs';

export const go = (h) => sheet(h, async (hand) => {
  await hand.tap(...(await hand.cell(1, 0)));
  await hand.key(' ');
});
