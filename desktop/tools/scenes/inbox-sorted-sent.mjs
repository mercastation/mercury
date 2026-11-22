// The Inbox sorted by when each thread was last sent rather than by priority, with no filter.
import { withDisplay } from './inbox-display.mjs';

export const go = (h) => withDisplay(h, { view: 'list', sort: 'updated', priorities: [], projects: [], updated: 'any' });
