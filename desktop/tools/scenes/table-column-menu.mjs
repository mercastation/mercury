// A header's caret pressed: the column's menu under it.
import { sheet } from './table.mjs';

export const go = (h) => sheet(h, async (hand) => {
  const [x, y] = await hand.header(1);
  await hand.move(x, y);
  await hand.tap(...(await hand.at('.tbl-th[data-hc="1"] .tbl-th-menu')));
});
