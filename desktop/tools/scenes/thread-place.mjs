// The waiting thread on a Mac set up for the cloud, with the button that says where it runs
// pressed: the panel under it, its two places and why (`PlaceButton` in
// `renderer/src/threads/PlaceButton.tsx`). The cloud takes both projects and runs nothing, so
// the thread's next reply starts there and Cloud is the place picked.
//
// The invented inbox has no cloud, and with none the button is not drawn: the page is fed the
// inbox with one, as the cloud button's scenes feed it (`inbox-cloud.mjs`).
import { open } from './thread.mjs';
import { CLOUDS } from './inbox-cloud.mjs';

export const dress = (snapshot) => ({ ...snapshot, cloud: CLOUDS.on });

export const pressPlace = async (h) => {
  await h.evaluate(`document.querySelector('.ts-place')?.click()`);
  await h.wait(400);
};

export const go = (h) => open(h, {
  unfold: false,
  waiting: true,
  then: async (h) => {
    await pressPlace(h);
    return { actions: 'place' };
  },
});
