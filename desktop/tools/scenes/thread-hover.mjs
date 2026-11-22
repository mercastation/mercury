// A conversation with the pointer resting on one of its controls: the thread as it first opens
// (`thread-folded`), or as one whose agent waits on an answer (`thread-options`), and then the
// pointer on what `at` finds. Each `thread-hover-*` scene names its control.
import { open } from './thread.mjs';
import { rest } from './pointer.mjs';

export async function hover(h, at, { waiting = false, unfold = false, ms = 200, ...more } = {}) {
  const fed = await open(h, { unfold, waiting, ...more });
  return { ...fed, ...(await rest(h, at, ms)) };
}
