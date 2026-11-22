// Home with a picture pasted into its box and no words: the picture drawn small in the row of
// attachments with its × in its corner, and Send lit, since a picture alone is something to send.
//
// The paste is the page's own: a clipboard event carrying the invented folder's widest picture
// as `image.png`, which is what a screenshot copied on a Mac arrives as, so the page names it
// `pasted-<number>.png`. The native window is handed the same picture's bytes and that name in
// the fixture (`pasted`), as its own paste would hold them.
import fs from 'node:fs';
import { plant } from '../pictures.mjs';

/** Pastes the PNG at `file` into the words under `box` (a selector), as ⌘V does with a
 * screenshot on the clipboard, and answers the name the page gave it. */
export async function pastePicture(h, box, file) {
  const bytes = fs.readFileSync(file).toString('base64');
  await h.evaluate(`(() => {
    const raw = Uint8Array.from(atob(${JSON.stringify(bytes)}), (c) => c.charCodeAt(0));
    const shot = new DataTransfer();
    shot.items.add(new File([raw], 'image.png', { type: 'image/png' }));
    const words = document.querySelector(${JSON.stringify(`${box} textarea.tc-text`)});
    words.focus();
    words.dispatchEvent(new ClipboardEvent('paste', { clipboardData: shot, bubbles: true, cancelable: true }));
  })()`);
  await h.wait(800);
  const names = await h.evaluate(`[...document.querySelectorAll(${JSON.stringify(`${box} .attach-thumb`)})].map((t) => t.title)`);
  return names.at(-1);
}

export async function go(h) {
  const planted = plant();
  const file = planted.files.find((f) => f.endsWith('checkout-wide.png'));
  const name = await pastePicture(h, '.hm-start', file);
  return { composing: '', pasted: [{ png: file, name }] };
}
