// `thread-sign-in` while "Not now" is on its way to the engine: every button of the card
// stands at half strength and takes no press.
import { signIn } from './thread-sign-in.mjs';

export const go = (h) => signIn(h, undefined, {
  bridge: { signInAnswer: '() => new Promise(() => {})' },
  then: async (h) => {
    await h.evaluate(`document.querySelector('.need-skip')?.click()`);
    return { signInPressed: ['skipped'], signInHangs: true };
  },
});
