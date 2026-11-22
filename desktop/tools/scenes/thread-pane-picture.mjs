// A picture pressed opens big over the window; its corner mark puts it beside the conversation
// instead (`onBeside` in renderer/src/components/ArtifactView.tsx), where the pane draws it whole
// in its card. The fixture says the picture was put beside (`docOpen`).
import { open } from './thread-pane.mjs';
import { pressPicture } from './thread-file-picture.mjs';

const SHIPPED = ({ thread }) => `**The receipt is drawn.**

The new receipt, small: designs/${thread}/receipt-small.png`;

export const go = (h) => open(h, {
  pressed: null,
  shipped: SHIPPED,
  then: async (h) => {
    const { pictureOpen } = await pressPicture(h, 0);
    await h.evaluate(`document.querySelector('.av-top .focus-mode-control, .av-top .artifact-expand')?.click()`);
    await h.wait(1500);
    return { docOpen: pictureOpen.path };
  },
});
