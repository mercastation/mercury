// The bar's Sort pressed: As the file has it, and every column.
import { sheet } from './table.mjs';

export const go = (h) => sheet(h, async (hand) => {
  await hand.tap(...(await hand.at('.tbl-bar .when-wrap:nth-of-type(2) .tbl-btn')));
});
