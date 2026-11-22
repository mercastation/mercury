// A conversation with the pointer resting on the model named in the folded reply box.
import { hover } from './thread-hover.mjs';

export const go = (h) => hover(h, '.dock-pill .ts-model');
