// Words typed into a tile's menu's Find a folder: the folders whose names hold them, flat, and
// nothing for the top of the project.
import { openFiles, press } from './files.mjs';

export const go = (h) => openFiles(h, { kept: true, project: 'Storefront', everything: true, then: async (h) => {
  await press(h, '.fp-tile .fp-more', 'More for checkout-wide.png');
  await h.evaluate(`(() => {
    const box = document.querySelector('.fp-menu-find');
    box.focus();
    const set = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, 'value').set;
    set.call(box, 'ca');
    box.dispatchEvent(new Event('input', { bubbles: true }));
  })()`);
  await h.wait(400);
  return { filesMenu: { file: 'storefront/checkout-wide.png', find: 'ca' } };
} });
