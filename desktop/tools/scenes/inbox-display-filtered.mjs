// The View and filters menu open on a display with filters on: Urgent and Low picked, one
// project picked, and Sort by Sent. The menu's foot says how many of the tab's threads show and
// offers Clear filters, and the button wears the accent dot (`isFiltered`).
import { openDisplay, withDisplay } from './inbox-display.mjs';

export const FILTERED = { view: 'list', sort: 'updated', priorities: ['urgent', 'low'], projects: ['storefront'], updated: 'any' };

export async function go(h) {
  const fed = await withDisplay(h, FILTERED);
  return { ...fed, ...(await openDisplay(h)) };
}
