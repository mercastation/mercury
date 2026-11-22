// The same answer as `thread-pictures` with no picture come yet: the place the page holds
// for each one while it is on its way.
import { open } from './thread.mjs';
import { SHIPPED } from './thread-pictures.mjs';

export const go = (h) => open(h, { unfold: false, shipped: SHIPPED, files: { arriving: true } });
