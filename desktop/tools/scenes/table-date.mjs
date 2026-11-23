// A date cell pressed once: its calendar drops from it, on the cell's month, its day filled.
import { sheet } from './table.mjs';

export const go = (h) => sheet(h, async (hand) => {
  await hand.tap(...(await hand.cell(1, 3)));
});
