// `thread-ship` over loose edits nobody committed: the offer says so over its options
// (`.opt-head-ask`), "This change was never committed.", and offers to commit it and ship.
import { ship } from './thread-ship.mjs';

export const go = (h) => ship(h, { stands: { committed: false, uncommitted: true, knowsYou: true } });
