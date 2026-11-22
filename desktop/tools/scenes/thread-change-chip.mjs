// A run's fold opened onto what it ran: the steps that edited or wrote a file the thread's change
// holds wear that file as a chip into the code (`.did-file` in renderer/src/components/Thread.tsx).
import { open, openRun } from './thread.mjs';
import { changeFor, hexId } from './change.mjs';

export const go = (h) => open(h, { unfold: true, runs: 'steps', files: {}, change: changeFor(), dress: hexId, then: (page) => openRun(page, 2) });
