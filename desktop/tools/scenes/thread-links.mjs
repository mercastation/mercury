// A conversation whose newest answer names files that are not pictures, each a way an agent
// writes one and each a kind the web window gives its own icon: a page, a note, a document
// behind words of the agent's own, a sheet, a source file in a code span, a full path into
// the project, a quoted name, an archive, an address on this Mac and one on the web.
import { open } from './thread.mjs';

const SHIPPED = ({ dir }) => `**Everything from this round is in the project.**

The page is pages/checkout.html and the notes are in notes/rounding.md, with the numbers in [the week's report](reports/week-32.pdf).

- The failed orders: exports/failed-orders.csv
- The fix itself: \`src/checkout/round-tax.ts\`
- The page again, in full: ${dir}/pages/checkout.html
- The plan, quoted: "notes/rounding.md", and linked: [notes/rounding.md](notes/rounding.md)
- All of it in one file: exports/round-7.zip

It runs at localhost:3000 while I am here, and the tax service's side is in [its release notes](https://example.com/tax/releases/4.2).`;

export const go = (h) => open(h, { unfold: false, shipped: SHIPPED, files: {} });
