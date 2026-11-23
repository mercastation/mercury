// A right press on a cell: it is picked and its menu opens at the pointer.
import { sheet } from './table.mjs';

export const go = (h) => sheet(h, async (hand) => {
  await hand.tap(...(await hand.cell(1, 0)), { right: true });
});
