// `thread-filed`, its earlier messages unfolded, with the threads filed while the agent's first
// answer was being written: the list hangs off that answer, the end of its turn, and not the
// conversation's foot (`filedOnTurn`, `tail`).
import { filed, PROPOSAL, FINISHED } from './thread-filed.mjs';

const MIN = 60_000;

export const go = (h) => filed(h, [PROPOSAL, FINISHED], { unfold: true, at: (row) => row.createdAt + 20 * MIN });
