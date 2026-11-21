// From Home to one open conversation: open the Inbox, then the first row of it, as Enter does.
import { go as toInbox } from './inbox.mjs';

export async function go(h) {
  await toInbox(h);
  const key = async (k, code) => {
    await h.call('Input.dispatchKeyEvent', { type: 'keyDown', key: k, code, windowsVirtualKeyCode: k === 'Enter' ? 13 : 40 });
    await h.call('Input.dispatchKeyEvent', { type: 'keyUp', key: k, code, windowsVirtualKeyCode: k === 'Enter' ? 13 : 40 });
  };
  await key('ArrowDown', 'ArrowDown');
  await h.wait(200);
  await key('Enter', 'Enter');
  await h.wait(1200);
}
