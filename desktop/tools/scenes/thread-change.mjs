// A thread whose run changed code: its head strip carries the code button, the code mark and
// the figures, beside the Files button (`codeButton` in renderer/src/components/Focus.tsx,
// `.ts-done.ts-code` in renderer/src/threads/summary.css). The change is invented
// (`./change.mjs`) and answered before the page loads, as the figures are read when the thread
// opens.
import { open } from './thread.mjs';
import { changeFor, hexId } from './change.mjs';

export const go = (h) => open(h, { unfold: false, files: {}, change: changeFor(), dress: hexId });
