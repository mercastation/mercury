// A one-time code the run asks for, with no sign-in before it: five digits typed and Send
// pressed, the warning under the box (`codeLooksWrong`), the way out under it.
import { needs, press, type } from './thread-needs.mjs';

export const go = (h) => needs(h, {
  said: 'The package is built.\n\nAGENTBOX-NEEDS: code npm | the one-time password for npm publish',
  then: async (h) => {
    await type(h, 0, 0, '48213');
    await press(h, 0, 'Send');
    await h.evaluate('document.activeElement?.blur()');
    return { cloudNeeds: { cards: { 0: { code: '48213', said: 'That is 5 digits; these codes are 6. Press Send again to send it anyway.' } } } };
  },
});
