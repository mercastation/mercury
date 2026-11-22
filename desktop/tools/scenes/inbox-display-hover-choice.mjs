// The View and filters menu open, the pointer resting on a choice it does not have on (High):
// its edge strengthens and its words brighten (`.th-pop .opts > button:hover`).
import { go as open } from './inbox-display.mjs';
import { rest } from './pointer.mjs';

export async function go(h) {
  const fed = await open(h);
  return { ...fed, ...(await rest(h, '.th-pop .line:nth-child(4) .opts > button:nth-child(2)')) };
}
