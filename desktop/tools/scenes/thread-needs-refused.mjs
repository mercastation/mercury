// A token the cloud did not take: its reason under the box, the token still in it.
import { needs, press, type } from './thread-needs.mjs';

export const TOKEN = 'sbp_0123456789abcdef0123';

export const go = (h) => needs(h, {
  bridge: { cloudKey: 'async () => ({ ok: false })' },
  then: async (h) => {
    await type(h, 0, 0, TOKEN);
    await press(h, 0, 'Save');
    await h.wait(300);
    await h.evaluate('document.activeElement?.blur()');
    return { cloudNeeds: { cards: { 0: { values: [TOKEN], fieldsSaid: 'The cloud did not take the key. Try again.' } } } };
  },
});
