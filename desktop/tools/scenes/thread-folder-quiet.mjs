// A folder on a line of its own under a message that draws a picture itself: the folder
// stays one line, its name, so the same pictures are not drawn twice. And the thread's own
// designs folder, which is named the same way.
import { open } from './thread.mjs';

const SHIPPED = ({ dir, thread }) => `**The tax line, and where the rounds are.**

![The tax line](designs/${thread}/tax-line.png)

${dir}/designs/rounds/

${dir}/designs/${thread}

Say which round, or ask for an eighth.`;

export const go = (h) => open(h, { unfold: false, shipped: SHIPPED, files: {} });
