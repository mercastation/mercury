// A conversation naming a folder of pictures (`thread-folder`), with the pointer resting on the
// folder's newest picture.
import { hover } from './thread-hover.mjs';

const SHIPPED = ({ dir }) => `**Seven rounds of the receipt, all in one folder.**

The seventh is the one I would ship. They are in:
${dir}/designs/rounds/

Say which round, or ask for an eighth.`;

export const go = (h) => hover(h, '.folder-preview-shot', { shipped: SHIPPED, files: {} });
