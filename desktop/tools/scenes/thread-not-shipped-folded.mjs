// `thread-not-shipped` as the thread first opens: the page leads with the agent's answer and
// the app's note is the quiet line under it. The card is at the conversation's foot, one
// press on "Show … earlier messages" away.
import { unshipped } from './thread-not-shipped.mjs';

export const go = (h) => unshipped(h, { unfold: false });
