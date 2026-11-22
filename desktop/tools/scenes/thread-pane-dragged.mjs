// The document beside the conversation, its grip dragged by the pointer to leave the pane less
// than half the body (`splitFromPointer`); the fixture says where it was left (`docSplit`).
import { open } from './thread-pane.mjs';

export const go = (h) => open(h, { split: 0.45 });
