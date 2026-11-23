// Words typed into the bar's search: the rows that hold them, and how many of how many.
import { sheet } from './table.mjs';

export const go = (h) => sheet(h, async (hand) => {
  await hand.tap(...(await hand.at('.tbl-search input')));
  await hand.type('dana');
});
