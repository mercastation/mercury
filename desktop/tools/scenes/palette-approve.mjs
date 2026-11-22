// The command bar over the Inbox when its first thread is the agent's question with options and
// no answer yet: Approve recommended leads its rows.
import { go as toInbox } from './inbox.mjs';
import { commandK, rowsOnScreen } from './palette.mjs';

export const FIRST = 'w-readme0';

export function dress(snapshot) {
  const items = snapshot.items.map((i) => (i.id !== FIRST ? i : {
    ...i, kind: 'question', status: 'blocked',
    result: 'Two ways to round the tax.\n\n## Options\n\n1. Round each line, then add them up\n2. Add the lines up, then round once (recommended)',
  }));
  return { ...snapshot, items };
}

export async function go(h) {
  await toInbox(h);
  await commandK(h);
  return { palette: '', paletteRows: await rowsOnScreen(h) };
}
