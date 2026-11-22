// The View and filters menu open, the pointer resting on the choice that is on (Priority, under
// Sort by): its edge goes to the strong line, its fill stays (`.th-pop .opts > button:hover`
// comes after `.on` and wins the edge).
import { go as open } from './inbox-display.mjs';
import { rest } from './pointer.mjs';

export async function go(h) {
  const fed = await open(h);
  return { ...fed, ...(await rest(h, '.th-pop .line:nth-child(2) .opts > button:nth-child(1)')) };
}
