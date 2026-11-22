// A folder on a line of its own, written out in full: its name, how many pictures it holds,
// its six newest, and a line for the rest. The planted folder has seven rounds, the seventh
// the newest, and one more in a folder of its own that does not count.
import { open } from './thread.mjs';

const SHIPPED = ({ dir }) => `**Seven rounds of the receipt, all in one folder.**

The seventh is the one I would ship. They are in:
${dir}/designs/rounds/

Say which round, or ask for an eighth.`;

export const go = (h) => open(h, { unfold: false, shipped: SHIPPED, files: {} });
