// `thread-sign-in-cloud` after "Page did not load after signing in?" was pressed and an address
// typed: the box for the address the browser ended on, which takes the keys, and Send.
import { signIn } from './thread-sign-in.mjs';
import { VERCEL } from './thread-sign-in-cloud.mjs';

export const go = (h) => signIn(h, VERCEL, {
  cloud: true,
  then: async (h) => {
    await h.evaluate(`document.querySelector('.need-also')?.click()`);
    await h.wait(300);
    await h.call('Input.insertText', { text: 'http://localhost:8976/callback?code=9f2' });
    return { signInPressed: ['also'], signInTyped: { landed: 'http://localhost:8976/callback?code=9f2' } };
  },
});
