// `thread-app-note` with the conversation unfolded: the app's note is the quiet line at its
// foot, where an agent's last word would wear "Result" or "Latest checkpoint" over it
// (`footOf` answering `app`).
import { noted, SHIPPED_NOTE } from './thread-app-note.mjs';

export const go = (h) => noted(h, SHIPPED_NOTE, { unfold: true });
