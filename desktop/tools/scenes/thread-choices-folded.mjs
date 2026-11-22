// A waiting thread's choices folded away by the little arrow in their corner: the strip goes,
// and the folded reply box carries how many there are and an arrow to bring them back
// (`ChoicesFold`, `ChoicesShow` in renderer/src/components/OneThingAtATime.tsx).
import { open } from './thread.mjs';

export const fold = async (h) => {
  await h.evaluate(`document.querySelector('.one-choices-fold')?.click()`);
  await h.wait(300);
  return { choicesFolded: true };
};

export const go = (h) => open(h, { unfold: false, waiting: true, then: fold });
