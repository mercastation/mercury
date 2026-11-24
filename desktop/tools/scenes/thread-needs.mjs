// A thread whose cloud run could not go on and ended its answer on what only the person can
// give (`AGENTBOX-NEEDS:` lines, `shared/cloud-needs.mts`): each line is a card where it stood
// (`NeedCard.tsx`, `.need`). The thread is open again, its newest answer the run's, nothing said
// to it since, on a Mac whose cloud is set up (`cloudKeys`, from `snap.cloud`).
//
// `needs` takes the answer's words (`said`), the keys the cloud already keeps (`keys`), whether
// the conversation is unfolded (`unfold`), a change to the row (`dress`), what the engine answers
// the cards' writes with, by the bridge's own names (`bridge`: a string of JavaScript for each
// function), and what is done to the cards once they are up (`then`), whose return goes in the
// fixture for the native window to do the same (`cloudNeeds`: each card's state by its place
// among the newest answer's cards, the keys just saved, the files held, what each file is).
import { open } from './thread.mjs';
import { CLOUDS } from './inbox-cloud.mjs';

export const SIGN_IN = '**I need one thing from you.**\n\nThe storage rules are written, and applying them needs the project\'s sign-in.\n\nAGENTBOX-NEEDS: sign-in supabase | to apply the storage rules to the shop\'s database';

export async function needs(h, { said = SIGN_IN, keys = [], cloud = true, unfold = false, dress = null, ledger = null, bridge = null, early = null, then = null, reply = undefined } = {}) {
  // The thread's row, for `then` (its id names what is held for it).
  let thread = null;
  // What the engine answers before the page is up: a card asks what its file is as it mounts.
  // Put on the bridge ahead of `thread.mjs`'s own, which keeps what it finds there.
  if (early) await h.call('Page.addScriptToEvaluateOnNewDocument', { source: `window.zero = Object.assign(window.zero ?? {}, { ${Object.entries(early).map(([name, fn]) => `${name}: ${fn}`).join(', ')} });` });
  return open(h, {
    unfold,
    shipped: said,
    ...(reply ? { reply } : {}),
    fromTab: 'Needs you',
    find: true,
    ...(ledger ? { ledger } : {}),
    dress: (row) => {
      row.status = 'open';
      thread = row;
      if (dress) dress(row);
    },
    fleet: (snapshot) => {
      if (cloud) snapshot.cloud = { ...CLOUDS.on, keys: keys.map((name) => ({ name, label: name })) };
    },
    then: async (h) => {
      if (bridge) await h.evaluate(`Object.assign(window.zero, { ${Object.entries(bridge).map(([name, fn]) => `${name}: ${fn}`).join(', ')} })`);
      return then ? (await then(h, thread)) ?? {} : {};
    },
  });
}

// Presses the first button of the `n`th card whose words start with `words`.
export const press = (h, n, words) => h.evaluate(`[...document.querySelectorAll('.need')[${n}].querySelectorAll('button')].find((b) => b.textContent.startsWith(${JSON.stringify(words)}))?.click()`);

// Types `text` into the `i`th box of the `n`th card.
export async function type(h, n, i, text) {
  await h.evaluate(`document.querySelectorAll('.need')[${n}].querySelectorAll('input')[${i}]?.focus()`);
  await h.wait(100);
  await h.call('Input.insertText', { text });
  await h.wait(100);
}

export const go = (h) => needs(h);
