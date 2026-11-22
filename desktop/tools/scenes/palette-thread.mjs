// ⌘K over a conversation that waits on the person (`thread-options`): the rows for the open
// thread, Send back to agent and Move to the other project among them, then the rest.
import { open } from './thread.mjs';
import { commandK, rowsOnScreen } from './palette.mjs';

/** ⌘K over the open thread, with `words` typed, handing the fixture what the bar drew. */
export const typedOver = (words) => async (h) => {
  await commandK(h);
  if (words) {
    await h.call('Input.insertText', { text: words });
    await h.wait(500);
  }
  return { palette: words, paletteRows: await rowsOnScreen(h) };
};

export const go = (h) => open(h, { unfold: false, waiting: true, then: typedOver('') });
