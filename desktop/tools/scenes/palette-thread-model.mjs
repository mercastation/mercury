// ⌘K over a waiting conversation with "model" typed: the three model rows, each with its agent.
import { open } from './thread.mjs';
import { typedOver } from './palette-thread.mjs';

export const go = (h) => open(h, { unfold: false, waiting: true, then: typedOver('model') });
