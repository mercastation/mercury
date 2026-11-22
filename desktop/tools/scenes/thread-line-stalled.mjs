// A stopped thread: the bar over the conversation, with the person's answer still waiting, and
// no line at its foot (`thread-line.mjs`, `.stalled-bar`).
import { line } from './thread-line.mjs';

export const go = (h) => line(h, 'stalled');
