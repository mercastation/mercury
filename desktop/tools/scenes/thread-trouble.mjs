// `inbox-trouble`'s row opened, as Enter opens it: which tasks are stopped and what each is
// waiting on (`troubleBody`), bare under the title, with no conversation and no reply box
// (`made` in `renderer/src/components/Focus.tsx`).
import { go as toInbox } from './inbox.mjs';

export { dress } from './inbox-trouble.mjs';

// The harness's bridge answers a conversation's compaction with a failure, which the pane draws
// under any thread it opens; a real app has none for a row with no conversation behind it.
export async function openTrouble(h) {
  await h.evaluate(`window.zero = { ...window.zero, compactionStatus: async () => null }`);
  await toInbox(h);
  await h.call('Input.dispatchKeyEvent', { type: 'keyDown', key: 'Enter', code: 'Enter', windowsVirtualKeyCode: 13 });
  await h.call('Input.dispatchKeyEvent', { type: 'keyUp', key: 'Enter', code: 'Enter', windowsVirtualKeyCode: 13 });
  await h.wait(1500);
  return { thread: 'trouble' };
}

export const go = (h) => openTrouble(h);
