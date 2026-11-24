// The ⋯ of a tile low on the page of every file: under 300 points from the window's foot, so its
// menu opens upward (`is-up`) and is then kept inside the page (`fitMenu`).
import { openFiles, press } from './files.mjs';

export const go = (h) => openFiles(h, { kept: true, project: 'Storefront', everything: true, then: async (h) => {
  await press(h, '.fp-tile .fp-more', 'More for week-32.pdf');
  return { filesMenu: { file: 'storefront/week-32.pdf' } };
} });
