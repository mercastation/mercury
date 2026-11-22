// The same picture opened big from its conversation, then its older version asked for with
// the version line's left arrow: "v1 of 2", and the button that goes back to the newest. The
// fixture says the arrow was pressed (`olderVersions`).
import { open } from './thread.mjs';
import { FOUND, SHIPPED } from './thread-files.mjs';
import { findsFiles, pressPicture } from './thread-file-picture.mjs';

export const go = (h) => open(h, {
  unfold: false,
  found: FOUND,
  shipped: SHIPPED,
  files: {},
  then: async (h) => {
    await findsFiles(h);
    const did = await pressPicture(h, 0);
    await h.evaluate(`document.querySelector('.av-chev[aria-label="Older version"]')?.click()`);
    await h.wait(1200);
    return { ...did, olderVersions: 1 };
  },
});
