// Two files chosen for the Trash: their rings filled and ticked, their faces' edges brighter, and
// the line saying how many and what they take.
import { openFiles, press } from './files.mjs';

export const go = (h) => openFiles(h, { kept: true, heavy: true, project: 'Storefront', everything: true, then: async (h) => {
  await press(h, '.fp-total');
  await press(h, '.fp-menu-item', 'Choose files to move to the Trash');
  await press(h, '.fp-open-file', 'Choose phone-tall.png');
  await press(h, '.fp-open-file', 'Choose checkout-wide.png');
  return { filesChosen: ['storefront/phone-tall.png', 'storefront/checkout-wide.png'] };
} });
