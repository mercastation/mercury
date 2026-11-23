// The change given the whole workspace by the pane's corners (`docFull`, which for a change keeps
// the pane's own full screen: `artifactMode` 'focus'). The fixture says so (`docFull`).
import { openChange } from './thread-change-open.mjs';
import { click } from './thread-pane.mjs';

export const go = (h) => openChange(h, {
  then: async (h) => {
    const at = await h.evaluate(`(() => { const b = document.querySelector('.doc-pane .focus-mode-control'); if (!b) return null; const r = b.getBoundingClientRect(); return [r.x + r.width / 2, r.y + r.height / 2]; })()`);
    if (!at) throw new Error('change scene: no corners mark');
    await click(h, at[0], at[1]);
    await h.call('Input.dispatchMouseEvent', { type: 'mouseMoved', x: 10, y: 400 });
    await h.wait(700);
    return { docFull: true };
  },
});
