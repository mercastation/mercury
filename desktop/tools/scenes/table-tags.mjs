// A tag cell pressed once: its list drops from it, the cell's own tag lit and ringed.
import { sheet } from './table.mjs';

export const go = (h) => sheet(h, async (hand) => {
  await hand.tap(...(await hand.cell(2, 1)));
});
