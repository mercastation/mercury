// A tag's own page, from the ⋯ of its row in a tag cell's list: its name, its colours, Delete.
import { sheet } from './table.mjs';

export const go = (h) => sheet(h, async (hand) => {
  await hand.tap(...(await hand.cell(2, 1)));
  await hand.tap(...(await hand.at('.tbl-pick-row.hi .tbl-pick-more')));
});
