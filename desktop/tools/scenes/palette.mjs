// ⌘K pressed on the Inbox: the command bar over the page, its box empty.
import { go as toInbox } from './inbox.mjs';

export const commandK = async (h) => {
  const key = { key: 'k', code: 'KeyK', windowsVirtualKeyCode: 75, modifiers: 4 };
  await h.call('Input.dispatchKeyEvent', { type: 'keyDown', ...key });
  await h.call('Input.dispatchKeyEvent', { type: 'keyUp', ...key });
  await h.wait(400);
};

export async function go(h) {
  await toInbox(h);
  await commandK(h);
  return { palette: '' };
}
