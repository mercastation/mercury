// The mode chip pressed on a thread that runs on Antigravity. The box offers Claude Code's
// modes only to a thread that reaches Claude Code (`claudeCode` in `DockComposer`), so this one
// offers the other list, Codex's three.
import { CHIP, click, on, reply } from './reply-chips.mjs';

export const dress = on('antigravity', 'bypassPermissions');
export const go = (h) => reply(h, { then: async (h) => { await click(h, CHIP.mode); return { actions: 'reply-mode' }; } });
