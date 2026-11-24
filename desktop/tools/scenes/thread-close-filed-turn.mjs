// `thread-close-filed` with the task filed on the agent's first turn, folded away as the thread
// opens: the close unfolds the conversation and brings the question, under that turn's list,
// onto the screen (`askTurnOf`, the unfolding effect in `ItemThread.tsx`).
import { closeFiled } from './thread-close-filed.mjs';
import { PROPOSAL, FINISHED } from './thread-filed.mjs';

const MIN = 60_000;

export const go = (h) => closeFiled(h, [PROPOSAL, FINISHED], { at: (row) => row.createdAt + 20 * MIN });
