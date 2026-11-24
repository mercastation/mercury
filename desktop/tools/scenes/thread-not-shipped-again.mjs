// A ship GitHub refused with the sign-in this computer had saved: "Sign in to GitHub again".
// Here GitHub's own sign-in did not start either (no `gh` to run it), so the card says so and
// opens the box for a token by itself, a token typed into it.
import { unshipped, REFUSED_SIGN_IN } from './thread-not-shipped.mjs';

export const go = (h) => unshipped(h, {
  note: REFUSED_SIGN_IN,
  bridge: { gitHubBrowserSignIn: 'async () => ({ ok: false })' },
  then: async (h) => {
    await h.evaluate(`[...document.querySelectorAll('.need .need-main')].find((b) => b.textContent.startsWith('Sign in'))?.click()`);
    await h.wait(300);
    await h.evaluate(`document.querySelector('.need .need-input')?.focus()`);
    await h.call('Input.insertText', { text: 'ghp_inventedtoken0000' });
    return { gitHubPressed: ['sign-in'], gitHubRefused: true, gitHubTyped: 'ghp_inventedtoken0000' };
  },
});
