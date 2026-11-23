// The change pressed open and refused by the engine: its reason where the change would be.
import { openChange } from './thread-change-open.mjs';
import { changeFor } from './change.mjs';

export const go = (h) => openChange(h, { change: changeFor({ refused: 'that change could not be read (an invented reason)' }) });
