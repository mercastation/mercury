// Home with the pointer resting on the box's project chip.
import { rest } from './pointer.mjs';

export const go = (h) => rest(h, '.hm-start [data-tour="project"] [data-trigger]');
