// Home with words typed into its box and one file attached through the paper clip: the row of
// attachments under the words, the file a chip with its name, its size and the ×.
//
// The file is the invented folder's report (`../pictures.mjs`), set on the box's own file input
// as the system's picker would set it (`DOM.setFileInputFiles`). The page in a browser tab has
// no path for a file, so it reads the file's bytes; the native window is handed the same file's
// path in the fixture (`attached`), as a drop or the picker hands it one.
import { plant } from '../pictures.mjs';
import { type, WORDS } from './home-typed.mjs';

/** Sets `files` (full paths) on the file input under `box` (a selector), as the picker does. */
export async function pickFiles(h, box, files) {
  await h.call('DOM.enable');
  const { result } = await h.call('Runtime.evaluate', { expression: `document.querySelector(${JSON.stringify(`${box} input[type=file]`)})` });
  if (!result?.objectId) throw new Error(`no file input under ${box}`);
  await h.call('DOM.setFileInputFiles', { files, objectId: result.objectId });
  await h.wait(600);
}

/** What the fixture says the box holds: each file by its path, as the native window attaches it. */
export const attached = (files) => files.map((path) => ({ path }));

export async function go(h) {
  const planted = plant();
  const file = planted.files.find((f) => f.endsWith('reports/week-32.pdf'));
  await type(h);
  await pickFiles(h, '.hm-start', [file]);
  return { composing: WORDS, attached: attached([file]) };
}
