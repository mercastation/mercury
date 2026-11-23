// Whole rows picked by their numbers: a press on one, Shift and a press two further down.
import { sheet } from './table.mjs';

export const go = (h) => sheet(h, async (hand) => {
  await hand.tap(...(await hand.rowNumber(1)));
  await hand.tap(...(await hand.rowNumber(3)), { shift: true });
});
