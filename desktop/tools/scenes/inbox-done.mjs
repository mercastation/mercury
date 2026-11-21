// From Home to the Inbox page's Done tab.
import { tab } from './inbox-progress.mjs';

export async function go(h) {
  await tab(h, 'Done');
}
