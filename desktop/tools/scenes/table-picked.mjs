// A cell of the sheet pressed: the ring on it, the pointer resting there (the row's way to open it).
import { sheet } from './table.mjs';

export const go = (h) => sheet(h, async (hand) => {
  await hand.tap(...(await hand.cell(1, 0)));
});
