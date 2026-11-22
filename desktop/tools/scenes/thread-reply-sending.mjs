// A reply sent on a thread with no run, opened from Home so the reply keeps the person on it: a
// second after Send, the reply is on the page under the answer, its head saying Z still takes
// it back, and the line about it stands in the strip (`answerWith`, `PendingSaid`).
import { open } from './thread.mjs';
import { press } from './thread-option-picked.mjs';

export const WORDS = 'Go with 1, and tell support first.';

/// Types `words` into the open reply box and sends them with ⌘↵.
export async function send(h, words) {
  await press(h, 'r', 'KeyR', 82);
  await h.wait(300);
  await h.call('Input.insertText', { text: words });
  await h.wait(200);
  await h.call('Input.dispatchKeyEvent', { type: 'keyDown', key: 'Enter', code: 'Enter', windowsVirtualKeyCode: 13, modifiers: 4 });
  await h.call('Input.dispatchKeyEvent', { type: 'keyUp', key: 'Enter', code: 'Enter', windowsVirtualKeyCode: 13, modifiers: 4 });
}

/// From the open thread back out, Home, and into the same thread from Home's first row.
export async function fromHome(h) {
  await press(h, 'Escape', 'Escape', 27);
  await h.wait(400);
  await h.evaluate(`(() => {
    const tabs = document.querySelectorAll('nav.workspace-tabs[aria-label="Inbox"] button.workspace-tab');
    const home = [...tabs].find((b) => b.dataset.tab === 'home') ?? tabs[0];
    home.click();
  })()`);
  await h.wait(900);
  await h.evaluate(`document.querySelector('.hm-row')?.click()`);
  await h.wait(1200);
}

export const go = (h) => open(h, {
  unfold: false, waiting: true,
  then: async (h) => {
    await fromHome(h);
    await send(h, WORDS);
    await h.wait(500);
    return { replySent: WORDS };
  },
});
