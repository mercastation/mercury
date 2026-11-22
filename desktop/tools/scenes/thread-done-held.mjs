// E pressed twice on a waiting thread: the first press only warns that its agent waits on an
// answer (`thread-done-toast`), the second marks it done. The page is back on the list, the
// thread is out of it, and for three seconds a line says so and that Z takes it back.
import { open } from './thread.mjs';
import { press } from './thread-option-picked.mjs';

export const go = (h) => open(h, {
  unfold: false, waiting: true,
  then: async (h) => {
    await press(h, 'e', 'KeyE', 69);
    await h.wait(400);
    await press(h, 'e', 'KeyE', 69);
    // E twice: the first is the once-ever warning's, and the window is told to press it twice.
    return { closed: 2 };
  },
});
