// The model's name on the folded reply box pressed: the box opens with the model menu already
// open over its chip (`openModel` in `renderer/src/components/Focus.tsx`, `ReplyModel`), the
// same menu as the New thread card's, rising out of the reply box.
import { open } from './thread.mjs';

export const go = (h) => open(h, {
  unfold: false,
  waiting: true,
  then: async (h) => {
    await h.evaluate(`document.querySelector('.dock-pill .ts-model')?.click()`);
    await h.wait(500);
    return { replying: '', actions: 'model' };
  },
});
