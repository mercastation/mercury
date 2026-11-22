// The command bar over the Inbox on a Mac with the cloud on, "cloud" typed: the toggle with ⌘C,
// what on means with the one on now said, and the other rows the word finds.
import { CLOUDS, withCloud } from './inbox-cloud.mjs';
import { commandK, rowsOnScreen } from './palette.mjs';

export const WORDS = 'cloud';

export async function go(h) {
  const fed = await withCloud(h, CLOUDS.on);
  await commandK(h);
  await h.call('Input.insertText', { text: WORDS });
  await h.wait(500);
  return { ...fed, palette: WORDS, paletteRows: await rowsOnScreen(h) };
}
