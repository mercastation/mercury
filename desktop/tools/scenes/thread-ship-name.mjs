// `thread-ship` over loose edits in a folder whose Git cannot name an author: "Git does not know
// who you are, so this change was never committed.", and the recommended press opens the reply
// box for a name and an email.
import { ship } from './thread-ship.mjs';

export const go = (h) => ship(h, { stands: { committed: false, uncommitted: true, knowsYou: false } });
