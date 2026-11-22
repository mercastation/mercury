// A thread whose agent named files on its way, some of them more than once: a picture and a
// note drawn again (`-v2`), two sheets and a report. The web window's head strip carries the
// Files button with how many there are, each counted once at its newest version
// (`ThreadFilesButton` in `renderer/src/threads/ThreadFiles.tsx`). The files are planted
// (`../pictures.mjs`), so both windows draw the picture and find the rest.
import { open } from './thread.mjs';

// The first answer: the first version of the picture and of the note, and a sheet.
export const FOUND = ({ thread }) => `**Found it: tax rounding breaks on carts over $1,000.**

The receipt adds the tax up before rounding it, as designs/${thread}/total-line.png shows. How it should round is in [the rounding note](notes/rounding.md), and every order it broke is in exports/failed-orders.csv.`;

// The last: both drawn again, another sheet and a report.
export const SHIPPED = ({ thread }) => `**Shipped, and support knows.**

The total sits on one line again, as in designs/${thread}/total-line-v2.png, and the note says why in notes/rounding-v2.md.

- The refunds, order by order: exports/refunds.csv
- The week with the fix in it: [the week's report](reports/week-32.pdf)`;

export const go = (h) => open(h, { unfold: false, found: FOUND, shipped: SHIPPED, files: {} });
