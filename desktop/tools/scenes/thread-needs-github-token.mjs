// `thread-needs-github` after "Paste a token instead": the facts with GitHub's token page, and
// the box for the token.
import { needs, press } from './thread-needs.mjs';
import { GITHUB } from './thread-needs-github.mjs';

export const go = (h) => needs(h, {
  said: GITHUB,
  then: async (h) => {
    await press(h, 0, 'Paste a token');
    await h.wait(200);
    return { cloudNeeds: { cards: { 0: { pasting: true } } } };
  },
});
