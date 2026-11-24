// `thread-needs-answered` after "Send it now" and "Send from this computer" on the film: "launch.mov
// sent", "This thread has it now." (`sentLate`).
import { press } from './thread-needs.mjs';
import { answered } from './thread-needs-answered.mjs';

export const go = (h) => answered(h, {
  bridge: { cloudSendFile: `async () => (${JSON.stringify({ ok: true, path: '/Users/you/Movies/launch.mov' })})` },
  then: async (h) => {
    await press(h, 2, 'Send it now');
    await h.wait(200);
    await press(h, 2, 'Send from');
    await h.wait(300);
    return { cloudNeeds: { cards: { 2: { sentLate: true } } } };
  },
});
