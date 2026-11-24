// `thread-sign-in` after "I've signed in" was pressed and the engine did not take it: the line
// under the buttons that says so.
import { signIn } from './thread-sign-in.mjs';

export const go = (h) => signIn(h, undefined, {
  bridge: { signInAnswer: 'async () => ({ ok: false })' },
  then: async (h) => {
    await h.evaluate(`[...document.querySelectorAll('.need-acts .need-btn')].find((b) => b.textContent.startsWith('I'))?.click()`);
    return { signInPressed: ['signed-in'], signInRefused: true };
  },
});
