// A design page named in a sentence and a note on a line of its own. The web window's rules
// (`renderer/src/message-artifacts.ts`) open such a file beside the conversation by itself.
// The reference page has no engine to find the file with, so no pane opens in its picture,
// and the two windows draw the same two mentions. What the real app does besides is not
// photographed here.
import { open } from './thread.mjs';

const SHIPPED = ({ thread }) => `**The plan is written down.**

It follows the page at designs/${thread}/checkout.html, and it is here:

notes/rounding.md`;

export const go = (h) => open(h, { unfold: false, shipped: SHIPPED, files: {} });
