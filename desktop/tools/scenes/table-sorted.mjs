// Sorted by a column from the Sort menu, which closes: the bar says so and the header carries
// its arrow.
import { sheet } from './table.mjs';

export const go = (h) => sheet(h, async (hand) => {
  await hand.tap(...(await hand.at('.tbl-bar .when-wrap:nth-of-type(2) .tbl-btn')));
  await hand.tap(...(await hand.row('Owner')));
});
