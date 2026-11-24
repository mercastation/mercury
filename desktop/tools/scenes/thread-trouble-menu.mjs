// `thread-trouble` with the dots at the right of its head strip pressed: the thread's own menu
// (`ThreadMenuList`), as it stands over the row the app makes itself. The rows are read into
// the fixture as `thread-actions` reads them (`actionRows`).
import { openTrouble } from './thread-trouble.mjs';
import { pressDots, rowsOnScreen } from './thread-actions.mjs';

export { dress } from './inbox-trouble.mjs';

export async function go(h) {
  const opened = await openTrouble(h);
  await pressDots(h);
  return { ...opened, actions: 'menu', actionRows: await rowsOnScreen(h) };
}
