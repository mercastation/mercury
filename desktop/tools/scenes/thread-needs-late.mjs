// `thread-needs-answered` after "Add it now" on the key: its box opens as it would live, but with
// no way out under it (`late`).
import { press } from './thread-needs.mjs';
import { answered } from './thread-needs-answered.mjs';

export const go = (h) => answered(h, {
  then: async (h) => {
    await press(h, 0, 'Add it now');
    await h.wait(200);
    await h.evaluate('document.activeElement?.blur()');
    return { cloudNeeds: { cards: { 0: { late: true } } } };
  },
});
