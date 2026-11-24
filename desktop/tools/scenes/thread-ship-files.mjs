// `thread-ship` over one loose file the thread's own words name, a note: not code to ship, so
// the offer asks in plain words, "Add refunds-for-support.md to the project?".
import { ship } from './thread-ship.mjs';
import { theChange } from './change.mjs';

const NOTE = 'notes/refunds-for-support.md';

const SAID = `**The note for support is written.**

It is in ${NOTE}: the 37 refunds wait until support has read it.`;

export const go = (h) => ship(h, {
  stands: { committed: false, uncommitted: true, knowsYou: true },
  said: SAID,
  make: (repo, at, o) => {
    const change = theChange(repo, at, o);
    const files = change.files.filter((f) => f.path === NOTE);
    return { ...change, files, plus: files.reduce((n, f) => n + f.plus, 0), minus: files.reduce((n, f) => n + f.minus, 0) };
  },
});
