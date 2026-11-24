// A sign-in of two variables (AWS): a box for each, Save after the last. The first was typed
// and Save pressed: "Paste the secret access key too." under the boxes.
import { needs, press, type } from './thread-needs.mjs';

// Invented, and shorter than a real one, so it has no real key's shape.
export const ID = 'AKIAINVENTED0000';

export const go = (h) => needs(h, {
  said: 'AGENTBOX-NEEDS: sign-in aws | to upload the build to the shop\'s bucket',
  then: async (h) => {
    await type(h, 0, 0, ID);
    await press(h, 0, 'Save');
    await h.evaluate('document.activeElement?.blur()');
    return { cloudNeeds: { cards: { 0: { values: [ID, ''], fieldsSaid: 'Paste the secret access key too.' } } } };
  },
});
