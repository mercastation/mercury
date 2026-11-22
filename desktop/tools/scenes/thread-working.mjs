// A thread an agent is on right now, opened from In progress: the conversation from the answer
// the person's reply followed on from, what the run has said since, and the line at the foot
// with the step it is on and how long it has been working.
import { open } from './thread.mjs';

export const go = (h) => open(h, { unfold: false, runs: 'working' });
