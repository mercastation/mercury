// The bar's Filter pressed: its menu, a heading and the values of each column it offers, with
// how many rows hold each.
import { sheet } from './table.mjs';

export const go = (h) => sheet(h, async (hand) => {
  await hand.tap(...(await hand.at('.tbl-bar .when-wrap:nth-of-type(1) .tbl-btn')));
});
