// From Home to the Inbox page's Later tab. The tab is only drawn while something is in it, so on
// an inbox with nothing put off this photographs the page it was already on.
import { tab } from './inbox-progress.mjs';

export async function go(h) {
  await tab(h, 'Later');
}
