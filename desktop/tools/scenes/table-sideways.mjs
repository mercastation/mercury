// The sheet scrolled sideways by a wheel: the row numbers and the first column stay put, a column
// half under them keeps its header's name in view, and the sheet settles on a column's edge
// (`scroll-snap-type: x proximity`) once the wheel stops.
import { sheet } from './table.mjs';

export const go = (h) => sheet(h, async (hand) => {
  const [x, y] = await hand.cell(2, 3);
  await hand.wheel(x, y, 150);
  const left = await h.evaluate(`document.querySelector('.tbl-scroll').scrollLeft`);
  return { scrolledTo: left };
});
