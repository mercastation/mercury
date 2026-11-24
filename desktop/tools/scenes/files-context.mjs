// A tile's menu opened by a right press on the tile: it stands at the pointer, not under the ⋯.
import { openFiles } from './files.mjs';
import { middleOf } from './pointer.mjs';

export const go = (h) => openFiles(h, { kept: true, project: 'Storefront', then: async (h) => {
  const [x, y] = await middleOf(h, '.fh-main .fh-tile');
  await h.call('Input.dispatchMouseEvent', { type: 'mouseMoved', x, y });
  await h.call('Input.dispatchMouseEvent', { type: 'mousePressed', x, y, button: 'right', clickCount: 1 });
  await h.call('Input.dispatchMouseEvent', { type: 'mouseReleased', x, y, button: 'right', clickCount: 1 });
  await h.wait(500);
  // Away again, so the picture is of the menu and not of a tile under the pointer.
  await h.call('Input.dispatchMouseEvent', { type: 'mouseMoved', x: 1270, y: 750 });
  // The page reads the pointer in whole points (`clientX`), and opens the menu four past it.
  return { filesMenu: { file: 'storefront/rounding-v2.md', at: [Math.floor(x) + 4, Math.floor(y) + 4] } };
} });
