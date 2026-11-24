// `thread-needs` with three asks on one answer: a sign-in, a file and a code. The facts are on
// the sign-in, the code says it waits for the sign-in, and the way out is under the code, the
// last ask still open. Nothing has told the page what the file is, so it says only where it is.
import { needs } from './thread-needs.mjs';

export const SEVERAL = [
  '**Three things, and the launch post goes out.**',
  '',
  'AGENTBOX-NEEDS: sign-in stripe | to read this month\'s revenue',
  'AGENTBOX-NEEDS: file ~/Movies/launch-film.mov | the cut is made from the original',
  'AGENTBOX-NEEDS: code npm | the one-time password for npm publish',
].join('\n');

export const go = (h) => needs(h, { said: SEVERAL });
