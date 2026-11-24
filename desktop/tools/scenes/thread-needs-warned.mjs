// A Vercel token that does not look like one, Save pressed: the warning under the box, and the
// button says "Save anyway" (`warned`).
import { needs, press, type } from './thread-needs.mjs';

export const go = (h) => needs(h, {
  said: 'AGENTBOX-NEEDS: sign-in vercel | to deploy the preview',
  then: async (h) => {
    await type(h, 0, 0, 'hunter2!');
    await press(h, 0, 'Save');
    await h.evaluate('document.activeElement?.blur()');
    return { cloudNeeds: { cards: { 0: { values: ['hunter2!'], warned: true, fieldsSaid: 'That does not look like the Vercel token. Press Save again to keep it anyway.' } } } };
  },
});
