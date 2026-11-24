// `thread-not-shipped` after "Sign in with GitHub" was pressed and GitHub's own sign-in
// started: the code to enter, Copy, "Open GitHub", and the line that says it ships again once
// approved.
import { unshipped } from './thread-not-shipped.mjs';

export const DEVICE = { code: 'A1B2-C3D4', url: 'https://github.com/login/device' };

export const go = (h) => unshipped(h, {
  bridge: { gitHubBrowserSignIn: `async () => (${JSON.stringify({ ok: true, ...DEVICE })})`, openArtifact: 'async () => ({ ok: true })' },
  then: async (h) => {
    await h.evaluate(`[...document.querySelectorAll('.need .need-main')].find((b) => b.textContent.startsWith('Sign in'))?.click()`);
    return { gitHubPressed: ['sign-in'], gitHubDevice: DEVICE };
  },
});
