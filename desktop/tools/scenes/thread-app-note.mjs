// A finished conversation on whose row the app then wrote a note of its own ("Shipped to main
// as …", written as `system`). The page still leads with the agent's answer, and the app's
// news is a quiet line under it (`.one-app-note`, `byApp` in `ItemThread.tsx`): never a
// checkpoint signed as the agent's.
//
// `noted` is the same conversation with any such note. `status` is what the app set the thread
// to with it (a ship that failed opens the thread again), `bridge` what the engine answers the
// page's writes with, by the bridge's own names (a string of JavaScript for each function), and
// `then` what is done once the page is up, whose return goes in the fixture.
import { open } from './thread.mjs';

const MIN = 60_000;

export const SHIPPED_NOTE = 'Shipped to main as 14c45bfb.';

export function noted(h, note, { unfold = false, status = null, fromTab = 'Done', bridge = null, then = null } = {}) {
  let at = 0;
  return open(h, {
    unfold,
    fromTab,
    find: true,
    ledger: (lines, row) => {
      at = row.updatedAt + MIN;
      return [
        ...lines,
        { id: row.id, ts: at, source: 'system', patch: { note } },
        ...(status ? [{ id: row.id, ts: at + 1, source: 'system', patch: { status } }] : []),
      ];
    },
    dress: (row) => {
      row.note = note;
      row.wrote = { ...row.wrote, note: { ts: at, source: 'system' } };
      if (status) {
        row.status = status;
        row.wrote.status = { ts: at + 1, source: 'system' };
      }
      row.updatedAt = at + 1;
    },
    then: async (h) => {
      if (bridge) await h.evaluate(`Object.assign(window.zero, { ${Object.entries(bridge).map(([name, fn]) => `${name}: ${fn}`).join(', ')} })`);
      return then ? then(h) : {};
    },
  });
}

export const go = (h) => noted(h, SHIPPED_NOTE);
