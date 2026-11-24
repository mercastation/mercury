// The command bar over the Inbox whose first row is the one the app makes itself when tasks have
// not been able to run (`inbox-trouble`): what ⌘K offers on that row.
import { go as toInbox } from './inbox.mjs';
import { commandK, rowsOnScreen } from './palette.mjs';

export { dress } from './inbox-trouble.mjs';

export async function go(h) {
  await toInbox(h);
  await commandK(h);
  return { palette: '', paletteRows: await rowsOnScreen(h) };
}
