// A script named in a code span, pressed: drawn in the pane beside the conversation as its text,
// numbered and coloured (`CodeFile`).
import { open } from './thread-pane.mjs';

export const go = (h) => open(h, { pressed: 'src/round-tax.ts' });
