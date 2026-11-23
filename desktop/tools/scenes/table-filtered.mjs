// A filter on: Meeting ticked in the Filter menu, which is then closed with Escape; the bar says
// what it is filtered by and how many rows of how many show.
import { sheet } from './table.mjs';

export const go = (h) => sheet(h, async (hand) => {
  await hand.tap(...(await hand.at('.tbl-bar .when-wrap:nth-of-type(1) .tbl-btn')));
  await hand.tap(...(await hand.row('Meeting')));
  await hand.key('Escape');
});
