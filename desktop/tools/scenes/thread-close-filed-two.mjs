// `thread-close-filed` with two tasks waiting: the question counts them in words, "both", and
// recommends keeping the thread to decide each one.
import { closeFiled } from './thread-close-filed.mjs';
import { MANY } from './thread-filed-many.mjs';

export const go = (h) => closeFiled(h, [MANY[0], MANY[1], MANY[3]]);
