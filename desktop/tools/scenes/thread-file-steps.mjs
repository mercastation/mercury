// Three pictures named in one sentence, the middle one pressed: the big view steps through the
// pictures of that answer, with the arrows at its sides (`steps` in `ArtifactView`).
import { open } from './thread.mjs';
import { findsFiles, pressPicture } from './thread-file-picture.mjs';

const SHIPPED = () => `**Three carts.**

Compare shots/cart-a.png, shots/cart-b.png and shots/cart-c.png before I pick one.`;

export const go = (h) => open(h, {
  unfold: false,
  shipped: SHIPPED,
  files: {},
  then: async (h) => {
    await findsFiles(h);
    return pressPicture(h, 1);
  },
});
