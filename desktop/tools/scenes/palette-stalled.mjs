// The command bar over the Inbox when the agent of its first thread, a finished one, stopped:
// Restart the agent, which Try again does not stand for on a finished thread.
import { go as toInbox } from './inbox.mjs';
import { commandK, rowsOnScreen } from './palette.mjs';
import { FIRST } from './palette-approve.mjs';

export function dress(snapshot) {
  return { ...snapshot, supervisor: { ...snapshot.supervisor, stalled: [FIRST] } };
}

export async function go(h) {
  await toInbox(h);
  await commandK(h);
  return { palette: '', paletteRows: await rowsOnScreen(h) };
}
