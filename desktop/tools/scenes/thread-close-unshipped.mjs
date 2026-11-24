// E pressed on a thread whose work never reached main (`shared/not-shipped.mts`: the ship queue
// wrote that it did not ship after the agent's last word): the close asks first, at the
// conversation's foot (`unshippedAsk`, `closeAskStrip`), "Have the agent ship it" recommended.
//
// `unshipped(h, { keys })`: `keys` are pressed after E, by the browser's names; the fixture
// says so for the native window (`closePressed`, `keysAfter`).
import { open } from './thread.mjs';
import { press } from './thread-option-picked.mjs';

const CODES = { ArrowDown: ['ArrowDown', 40], ArrowUp: ['ArrowUp', 38], Enter: ['Enter', 13] };

export async function pressKeys(h, keys) {
  for (const key of keys) {
    const [code, vk] = CODES[key] ?? [`Digit${key}`, 48 + Number(key)];
    await press(h, key, code, vk);
    await h.wait(150);
  }
}

export function unshipped(h, { keys = [] } = {}) {
  return open(h, {
    unfold: false,
    fromTab: 'Needs you',
    dress: (row) => {
      row.status = 'blocked';
      row.note = 'It did not ship, and it went back to its agent: the checkout tests on main are red';
      row.wrote.note = { ts: row.wrote.result.ts + 60_000, source: 'system' };
      row.wrote.status = { ts: row.wrote.result.ts + 60_000, source: 'system' };
    },
    then: async (h) => {
      await press(h, 'e', 'KeyE', 69);
      await h.wait(300);
      await pressKeys(h, keys);
      return { closePressed: true, ...(keys.length ? { keysAfter: keys } : {}) };
    },
  });
}

export const go = (h) => unshipped(h);
