// `thread-needs-several` after the film was sent while Stripe was still open: the engine held it
// (`hold`), and its card says it is ready and goes with what is left (`held`, `othersLeft`).
import { needs, press } from './thread-needs.mjs';
import { SEVERAL } from './thread-needs-several.mjs';

const WORDS = 'I sent ~/Movies/launch-film.mov. It is at that same path on your machine now.';

export const go = (h) => needs(h, {
  said: SEVERAL,
  bridge: { cloudSendFile: `async () => (${JSON.stringify({ ok: true, path: '/Users/you/Movies/launch-film.mov', held: true, words: WORDS })})` },
  then: async (h, row) => {
    await press(h, 1, 'Send from');
    await h.wait(300);
    // Sent, the keys go on to the next ask still open: Stripe's box (`focusNextAsk`).
    return { cloudNeeds: { held: { [row.id]: [{ need: 'file:~/Movies/launch-film.mov', words: WORDS }] }, cards: { 0: { focused: 0 } } } };
  },
});
