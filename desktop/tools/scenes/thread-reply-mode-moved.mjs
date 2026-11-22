// A waiting thread on Claude Code, its options over the box, a line typed, the next reply moved
// to Antigravity's model at High (the line under the words says it starts fresh), and the mode
// chip pressed: the modes stay Claude Code's, as they follow the agent the thread is on now.
import { CHIP, click, on, reply } from './reply-chips.mjs';

const WORDS = 'Ship it.';

/** Presses the button of the open menu whose words include `words`. */
const row = async (h, words) => {
  const found = await h.evaluate(`(() => {
    const b = [...document.querySelectorAll('.tc-menu button, .tc-drop button')].find((b) => b.textContent.includes(${JSON.stringify(words)}));
    if (!b) return false;
    b.click();
    return true;
  })()`);
  if (!found) throw new Error(`no row says ${words}`);
  await h.wait(300);
};

const MODEL_CHIP = '.dock-bar .dock-chips .model-pick .tc-chip';

export const dress = on('claude', 'bypassPermissions');
export const go = (h) => reply(h, {
  waiting: true,
  then: async (h) => {
    await h.call('Input.insertText', { text: WORDS });
    await click(h, MODEL_CHIP);
    await row(h, 'All models');
    await row(h, 'Gemini 9 Ember');
    await click(h, MODEL_CHIP);
    await row(h, 'High');
    await click(h, MODEL_CHIP);
    await click(h, CHIP.mode);
    return { replying: WORDS, replyPick: { engine: 'antigravity', model: 'gemini-9-ember', effort: 'high' }, actions: 'reply-mode' };
  },
});
