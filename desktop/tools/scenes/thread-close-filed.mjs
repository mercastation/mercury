// E pressed on a thread with a task it filed still waiting for a yes (`closeAsk`): the question
// stands under the list of what it filed, at the foot where the newest turn's list is
// (`askHere`, `.close-ask-at-task`).
//
// `closeFiled(h, kids, opts)` is `filed` from ./thread-filed.mjs with E pressed once it is open.
import { filed, PROPOSAL, FINISHED } from './thread-filed.mjs';
import { press } from './thread-option-picked.mjs';

export function closeFiled(h, kids, opts = {}) {
  return filed(h, kids, {
    ...opts,
    then: async (h) => {
      await press(h, 'e', 'KeyE', 69);
      await h.wait(600);
      return { closePressed: true };
    },
  });
}

export const go = (h) => closeFiled(h, [PROPOSAL, FINISHED]);
