// A cell being typed into: pressed, Enter, and words added after its own.
import { sheet } from './table.mjs';

export const go = (h) => sheet(h, async (hand) => {
  await hand.tap(...(await hand.cell(2, 0)));
  await hand.key('Enter');
  await hand.type(' Corporation');
});
