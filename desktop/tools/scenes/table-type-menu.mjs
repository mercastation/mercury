// A column's menu on its page of types: the column's name to go back, and the six types, its
// own ticked.
import { sheet } from './table.mjs';

export const go = (h) => sheet(h, async (hand) => {
  const [x, y] = await hand.header(1);
  await hand.move(x, y);
  await hand.tap(...(await hand.at('.tbl-th[data-hc="1"] .tbl-th-menu')));
  await hand.tap(...(await hand.row('Type')));
});
