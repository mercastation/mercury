// A thread whose run ended, left a change and asked nothing: the app asks on its behalf
// (`renderer/src/ship-offer.ts`), in the strip over the reply box where an agent's options
// stand. Here the change has a commit on it, so the offer is "Ship it" (recommended) and
// "Discard the change", bare: the page's own headline is already the question.
//
// `ship(h, { stands, make, said, then })`: `stands` is where the run's folder stands
// (`committed`, `uncommitted`, `knowsYou` on the saved change, `main/git-change.mts`), `make`
// builds another change on the planted folder, `said` is the agent's last answer, `then` what
// is done once the thread is open. The thread's agent finished and waits on the person
// (`blocked`), and nothing it said offers options of its own.
import { open } from './thread.mjs';
import { changeFor, hexId, theChange } from './change.mjs';

export function ship(h, { stands = {}, make = theChange, said, then = null } = {}) {
  return open(h, {
    unfold: false,
    files: {},
    ...(said ? { shipped: said } : {}),
    fromTab: 'Needs you',
    change: changeFor({ make: (repo, at, o) => ({ ...make(repo, at, o), ...stands }) }),
    dress: (row) => {
      hexId(row);
      row.status = 'blocked';
      row.wrote.status = { ts: row.wrote.result.ts, source: 'agent' };
    },
    then,
  });
}

export const go = (h) => ship(h, { stands: { committed: true, uncommitted: false } });
