// An answer asking for a key, a sign-in, a file and a code, answered in words since: each card
// says only what it asked, "Answered below.", with "Add it now", "Connect it now" and "Send it
// now"; a code has run out and offers nothing (`!taking`). The page is the conversation, where
// the person's words stand under the answer.
import { needs } from './thread-needs.mjs';

const MIN = 60_000;

export const FOUR = [
  'Four things, then the launch goes out.',
  '',
  'AGENTBOX-NEEDS: key ELEVENLABS_API_KEY | ElevenLabs API key, from elevenlabs.io/app/settings/api-keys',
  'AGENTBOX-NEEDS: sign-in sentry | to read the errors',
  'AGENTBOX-NEEDS: file ~/Movies/launch.mov | to cut it',
  'AGENTBOX-NEEDS: code npm | to publish',
].join('\n');

export const answered = (h, more = {}) => needs(h, {
  said: FOUR,
  ledger: (lines, row) => [...lines, { id: row.id, ts: row.updatedAt + MIN, source: 'founder', patch: { answer: 'Do it from my computer.', status: 'open' } }],
  dress: (row) => {
    row.answer = 'Do it from my computer.';
    row.wrote = { ...row.wrote, answer: { ts: row.updatedAt + MIN, source: 'founder' } };
    row.updatedAt += MIN;
  },
  ...more,
});

export const go = (h) => answered(h);
