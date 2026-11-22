// A sheet named in a sentence, pressed: drawn in the pane beside the conversation as its table
// (`TableView` in the pane's mode).
import { open } from './thread-pane.mjs';

export const go = (h) => open(h, { pressed: 'exports/orders.csv' });
