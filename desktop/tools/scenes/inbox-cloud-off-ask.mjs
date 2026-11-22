// ⌘C on the Inbox while the cloud is on: turning it off asks first (`CloudOffAsk`), as both ⌘C
// and ⌘K were pressed by accident.
import { CLOUDS, withCloud } from './inbox-cloud.mjs';
import { MONTH } from './inbox-cloud-menu.mjs';

export async function go(h) {
  const fed = await withCloud(h, { ...CLOUDS.on, month: MONTH });
  for (const type of ['keyDown', 'keyUp']) {
    await h.call('Input.dispatchKeyEvent', { type, key: 'c', code: 'KeyC', windowsVirtualKeyCode: 67, modifiers: 4 });
  }
  await h.wait(400);
  return { ...fed, cloudOffAsk: true };
}
