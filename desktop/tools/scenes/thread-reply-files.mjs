// The reply box with words in it, a picture pasted into it and a file picked with its paper clip:
// the row of what it carries between the words and its bar, the picture drawn small with its ×
// in its corner, the file a chip with its name, its size and the ×.
//
// As on Home (`home-attached`, `home-attached-picture`): the paste is the page's own clipboard
// event carrying the invented folder's widest picture as `image.png`, and the file is the
// folder's report set on the box's own file input. The native window is handed the same picture's
// bytes and name and the report's path, in the order they came (`replyFiles`).
import fs from 'node:fs';
import { open } from './thread.mjs';
import { press } from './thread-option-picked.mjs';
import { plant } from '../pictures.mjs';
import { attached, pickFiles } from './home-attached.mjs';

export const WORDS = 'Use this layout for the receipt, and check it against the report.';

/** Pastes the PNG at `file` into the reply box's words, as ⌘V does with a screenshot on the
 * clipboard, and answers the name the page gave it. */
export async function pasteIntoTheReply(h, file) {
  const bytes = fs.readFileSync(file).toString('base64');
  await h.evaluate(`(() => {
    const raw = Uint8Array.from(atob(${JSON.stringify(bytes)}), (c) => c.charCodeAt(0));
    const shot = new DataTransfer();
    shot.items.add(new File([raw], 'image.png', { type: 'image/png' }));
    const words = document.querySelector('textarea.dock-input');
    words.focus();
    words.dispatchEvent(new ClipboardEvent('paste', { clipboardData: shot, bubbles: true, cancelable: true }));
  })()`);
  await h.wait(800);
  const names = await h.evaluate(`[...document.querySelectorAll('.attach-thumb')].map((t) => t.title)`);
  return names.at(-1);
}

/** R, the words, and then what `carry` puts in the box, for the fixture to say. */
export const replyCarrying = (carry) => async (h) => {
  await press(h, 'r', 'KeyR', 82);
  await h.wait(300);
  await h.call('Input.insertText', { text: WORDS });
  return { replying: WORDS, ...(await carry(h)) };
};

export const go = (h) => open(h, {
  unfold: false,
  then: replyCarrying(async (h) => {
    const planted = plant();
    const picture = planted.files.find((f) => f.endsWith('checkout-wide.png'));
    const report = planted.files.find((f) => f.endsWith('reports/week-32.pdf'));
    const name = await pasteIntoTheReply(h, picture);
    await pickFiles(h, '.dock-card', [report]);
    return { replyFiles: [{ png: picture, name }, ...attached([report])] };
  }),
});
