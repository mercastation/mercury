// One conversation as it first opens: its newest answer alone, the rest folded behind the line
// under the title. The same thread and the same words as the `thread` scene.
import { open } from './thread.mjs';

export const go = (h) => open(h, { unfold: false });
