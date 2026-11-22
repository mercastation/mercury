// A filter that keeps none of Needs you: High picked, and the invented inbox has no thread at
// High. The page says the filter holds them back and offers to clear it (`FilteredEmpty`).
import { withDisplay } from './inbox-display.mjs';

export const go = (h) => withDisplay(h, { view: 'list', sort: 'priority', priorities: ['high'], projects: [], updated: 'any' });
