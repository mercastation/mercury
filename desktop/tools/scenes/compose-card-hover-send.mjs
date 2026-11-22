// The New thread card with words typed and the pointer resting on Send, which has them to send.
import { go as card } from './compose-card.mjs';
import { rest } from './pointer.mjs';

export async function go(h) {
  const fed = await card(h);
  return { ...fed, ...(await rest(h, '.tc-card .tc-send-main')) };
}
