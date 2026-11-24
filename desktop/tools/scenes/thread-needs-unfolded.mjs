// `thread-needs-several` with the whole conversation unfolded: the cards in the answer at the
// conversation's foot, live as on the page that leads with it.
import { needs } from './thread-needs.mjs';
import { SEVERAL } from './thread-needs-several.mjs';

export const go = (h) => needs(h, { said: SEVERAL, unfold: true });
