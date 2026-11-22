// The newest answer sets its plan on a line of its own, so the pane opens beside it by itself
// (`besideDocument` in renderer/src/message-artifacts.ts); nothing is pressed. The page looks
// for the file as the thread opens, before this scene has answered its finder, so the scene
// leaves the thread and opens it again, as Escape and Enter do.
import { open } from './thread-pane.mjs';

const SHIPPED = `**The plan is written down.**

The refunds are at its foot, and support has the note:

designs/receipt-plan.md`;

const key = async (h, key, code, vk) => {
  await h.call('Input.dispatchKeyEvent', { type: 'keyDown', key, code, windowsVirtualKeyCode: vk });
  await h.call('Input.dispatchKeyEvent', { type: 'keyUp', key, code, windowsVirtualKeyCode: vk });
};

export const go = (h) => open(h, {
  pressed: null,
  shipped: SHIPPED,
  then: async (h) => {
    await key(h, 'Escape', 'Escape', 27);
    await h.wait(600);
    await key(h, 'Enter', 'Enter', 13);
    await h.wait(2000);
    return { docAuto: true };
  },
});
