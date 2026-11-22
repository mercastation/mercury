// The reply box of `thread-reply-files` folded shut by a press on nothing in the page, as the
// web window folds it (`away` in `renderer/src/App.tsx`): one line saying the words written in
// it, dimmer, and its two files counted beside them (`foldedReply`). The fixture says the box
// was folded (`folded`), over the words and files it held.
import { go as carrying } from './thread-reply-files.mjs';

export const go = async (h) => {
  const did = await carrying(h);
  // A press on the empty page beside the answer, above the dock (`.focus-dock` is something with a job).
  await h.call('Input.dispatchMouseEvent', { type: 'mousePressed', x: 1100, y: 300, button: 'left', clickCount: 1 });
  await h.call('Input.dispatchMouseEvent', { type: 'mouseReleased', x: 1100, y: 300, button: 'left', clickCount: 1 });
  await h.wait(400);
  return { ...did, folded: true };
};
