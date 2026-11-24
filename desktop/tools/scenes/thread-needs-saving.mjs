// A token on its way to the cloud: "Saving", held, the box hiding what it holds.
import { needs, press, type } from './thread-needs.mjs';
import { TOKEN } from './thread-needs-refused.mjs';

export const go = (h) => needs(h, {
  bridge: { cloudKey: '() => new Promise(() => {})' },
  then: async (h) => {
    await type(h, 0, 0, TOKEN);
    await press(h, 0, 'Save');
    await h.evaluate('document.activeElement?.blur()');
    return { cloudNeeds: { cards: { 0: { values: [TOKEN], saving: true } } } };
  },
});
