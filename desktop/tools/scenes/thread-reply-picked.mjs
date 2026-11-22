// High picked in the priority chip's menu, then Plan in the mode chip's: both chips say the
// pick, and the line in the strip says what the next reply may do for this thread
// (`announce` in `DockComposer`).
import { CHIP, click, on, reply } from './reply-chips.mjs';

const pick = async (h, words) => {
  const found = await h.evaluate(`(() => {
    const b = [...document.querySelectorAll('.prio-menu .prio-menu-row')].find((b) => b.textContent.trim() === ${JSON.stringify(words)});
    if (!b) return false;
    b.click();
    return true;
  })()`);
  if (!found) throw new Error(`no row says ${words}`);
  await h.wait(300);
};

export const dress = on('claude', 'bypassPermissions');
export const go = (h) => reply(h, {
  then: async (h) => {
    await click(h, CHIP.priority);
    await pick(h, 'High');
    await click(h, CHIP.mode);
    await pick(h, 'Plan');
    return { replyPicked: { priority: 'high', mode: 'plan' } };
  },
});
