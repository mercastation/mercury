// The mode chip pressed on a thread that runs on Codex: Codex's own three (`CODEX_MODE_ORDER`),
// on Auto, as a Codex worker runs with nothing picked.
import { CHIP, click, on, reply } from './reply-chips.mjs';

export const dress = on('codex', 'bypassPermissions');
export const go = (h) => reply(h, { then: async (h) => { await click(h, CHIP.mode); return { actions: 'reply-mode' }; } });
