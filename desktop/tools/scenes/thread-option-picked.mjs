// The same waiting thread with the arrow pressed once: the first option is the one Enter sends.
import { open } from './thread.mjs';

export const press = async (h, key, code, vk) => {
  await h.call('Input.dispatchKeyEvent', { type: 'keyDown', key, code, windowsVirtualKeyCode: vk });
  await h.call('Input.dispatchKeyEvent', { type: 'keyUp', key, code, windowsVirtualKeyCode: vk });
};

export const go = (h) => open(h, {
  unfold: false, waiting: true,
  then: async (h) => { await press(h, 'ArrowDown', 'ArrowDown', 40); return { option: 1 }; },
});
