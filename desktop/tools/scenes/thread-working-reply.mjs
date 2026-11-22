// A thread an agent is on, its reply box opened with R and nothing typed: Stop stands where Send
// would (`stop` in `DockComposer`).
import { open } from './thread.mjs';
import { press } from './thread-option-picked.mjs';

export const go = (h) => open(h, {
  unfold: false, runs: 'working',
  then: async (h) => { await press(h, 'r', 'KeyR', 82); return { replying: '' }; },
});
