// A conversation that draws pictures (`thread-pictures`), with the pointer resting on the first.
import { hover } from './thread-hover.mjs';
import { SHIPPED } from './thread-pictures.mjs';

export const go = (h) => hover(h, '.inline-image-open .inline-image', { shipped: SHIPPED, files: {} });
