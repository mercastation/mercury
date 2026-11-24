// The app's own note in the middle of a conversation: the thread shipped, the app said so on
// its row, and the person wrote again after that. Unfolded, the note is a quiet line where it
// was written, with no name over it and no checkpoint's tag (`e.says === 'app'` in
// `Thread.tsx`), between the agent's answer and the person's next words.
import { open } from './thread.mjs';
import { SHIPPED_NOTE } from './thread-app-note.mjs';

const SEC = 1000;
const ASKED_AGAIN = 'Thanks. Did the refunds go out too?';
const ANSWERED = 'All 37 went out this morning, after support had the note.';

export const go = (h) => {
  let at = 0;
  return open(h, {
    unfold: true,
    fromTab: 'Needs you',
    find: true,
    ledger: (lines, row) => {
      at = row.updatedAt + 20 * SEC;
      return [
        ...lines,
        { id: row.id, ts: at, source: 'system', patch: { note: SHIPPED_NOTE } },
        { id: row.id, ts: at + 40 * SEC, source: 'founder', patch: { answer: ASKED_AGAIN, status: 'open' } },
        { id: row.id, ts: at + 50 * SEC, source: 'agent', epoch: 4, claim: { holder: 'mcp-40140', leaseUntil: at + 31 * 60 * SEC }, patch: { status: 'claimed' } },
        { id: row.id, ts: at + 130 * SEC, source: 'agent', epoch: 4, patch: { status: 'done', result: ANSWERED } },
      ];
    },
    dress: (row) => {
      row.note = SHIPPED_NOTE;
      row.answer = ASKED_AGAIN;
      row.result = ANSWERED;
      row.wrote = {
        ...row.wrote,
        note: { ts: at, source: 'system' },
        answer: { ts: at + 40 * SEC, source: 'founder' },
        result: { ts: at + 130 * SEC, source: 'agent' },
        status: { ts: at + 130 * SEC, source: 'agent' },
      };
      row.updatedAt = at + 130 * SEC;
    },
  });
};
