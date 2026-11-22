// The Inbox under the filters of `inbox-display-filtered`, its menu closed: the rows the filters
// keep, in the order Sort by Sent puts them, the tabs counting what shows, and the dot on the
// display button.
import { withDisplay } from './inbox-display.mjs';
import { FILTERED } from './inbox-display-filtered.mjs';

export const go = (h) => withDisplay(h, FILTERED);
