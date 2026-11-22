// The View and filters menu over Done, whose order Sort by does not change: its line says so.
// Drawn here with `--tab done`.
import { DEFAULT, openDisplay } from './inbox-display.mjs';
import { tab } from './inbox-progress.mjs';

export async function go(h) {
  await tab(h, 'Done');
  return { display: DEFAULT, ...(await openDisplay(h)) };
}
