// The reply box's first two chips, the priority and what the message may do, pressed open
// (`PriorityPicker` and `ModePicker` in renderer/src/components/Focus.tsx's `DockComposer`).
// The thread is the first of the Inbox, opened, with R pressed.
import { open } from './thread.mjs';
import { press } from './thread-option-picked.mjs';

const ENGINES = [
  { id: 'claude', label: 'Claude Code', word: 'Claude Code' },
  { id: 'codex', label: 'Codex', word: 'Codex' },
  { id: 'antigravity', label: 'Antigravity', word: 'Antigravity' },
];

// The models the page reads with its settings (`zero:settings-read`), invented: Codex's and
// Antigravity's are never in the snapshot.
export const SETTINGS = {
  ok: true,
  workspace: {
    codexModels: [{ id: 'gpt-6-astra', label: 'GPT-6 Astra', levels: ['low', 'medium', 'high', 'xhigh'], defaultLevel: 'medium' }],
    codexModelDefault: 'gpt-6-astra',
    antigravityModels: [{ id: 'gemini-9-ember', label: 'Gemini 9 Ember', levels: ['low', 'medium', 'high'] }],
  },
};

/** The invented inbox on a Mac offering three agents, every thread on `engine`, every project
 *  set to `permission` (`config.permission`, which the chip reads as the project's own mode). */
export const on = (engine, permission = 'auto') => (snapshot) => ({
  ...snapshot,
  config: { ...snapshot.config, permission: { '': permission } },
  engines: { choices: ENGINES, workspace: 'claude', byItem: Object.fromEntries(snapshot.items.map((i) => [i.id, engine])) },
});

/** The chip's button: the priority's is the first of the box's chips, the mode's wears `mode-pick`. */
export const CHIP = {
  priority: '.dock-bar .dock-chips > .compose-word-wrap:not(.mode-pick) > .tc-chip',
  mode: '.dock-bar .dock-chips > .mode-pick > .tc-chip',
};

export const click = async (h, selector) => {
  const found = await h.evaluate(`(() => { const b = document.querySelector(${JSON.stringify(selector)}); if (!b) return false; b.click(); return true; })()`);
  if (!found) throw new Error(`nothing answers ${selector}`);
  await h.wait(300);
};

/** The page answered its settings from here on, loaded again (`withSettings`). */
const settingsAnswered = (h) => h.call('Page.addScriptToEvaluateOnNewDocument', {
  source: `(() => { const answer = ${JSON.stringify(SETTINGS)}; const once = () => { if (window.zero) window.zero.settingsRead = async () => answer; }; once(); setTimeout(once, 0); })();`,
});

/** Opens the thread, R, then `then` with the reply box up. `waiting` leaves the agent's options
 *  over the box. What the box was given to the native window: `replying`, and whatever `then`
 *  returns besides. */
export async function reply(h, { waiting = false, then = null } = {}) {
  await settingsAnswered(h);
  return open(h, {
    unfold: false,
    waiting,
    then: async (h) => {
      await press(h, 'r', 'KeyR', 82);
      await h.wait(300);
      const did = then ? (await then(h)) ?? {} : {};
      return { replying: '', settings: SETTINGS, ...did };
    },
  });
}
