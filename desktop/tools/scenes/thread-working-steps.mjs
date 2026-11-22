// The line at the foot of a working thread pressed open: the steps the run took since it last
// said anything, the newest five, and the call running now, each a line that opens onto its
// command (`Live.tsx`, expanded). The fixture says the line was open (`liveOpen`).
import { open } from './thread.mjs';

/** Presses the working line open. */
export async function unfold(h) {
  await h.evaluate(`document.querySelector('.live-disclosure')?.click()`);
  await h.wait(600);
  return { liveOpen: true };
}

export const go = (h) => open(h, { unfold: false, runs: 'working', then: unfold });
