// The person's account menu (`AccountMenu` in renderer/src/team/account-menu.tsx), opened by a
// press on their own row at the sidebar's foot, for the `home-team-account*` scenes. Everyone
// here is invented (`home-team.mjs`).
//
// The sidebar opens the only way the web opens it now: the pointer comes onto it from the page
// beside it and rests there 900 ms (`restOnTheSidebar`); kept open (`kept`), it stands open from
// the start, as `pointAtOpen` in usage.mjs keeps it. Then the pointer goes onto the person's row
// in two steps and presses it; `then` takes the pointer on to what the scene shows next. Every
// move and press is handed to the native window too (`script`, `at`).
import { dress as dressTeam } from './home-team.mjs';

const move = (h, x, y) => h.call('Input.dispatchMouseEvent', { type: 'mouseMoved', x, y });
const down = (h, x, y) => h.call('Input.dispatchMouseEvent', { type: 'mousePressed', x, y, button: 'left', clickCount: 1 });
const up = (h, x, y) => h.call('Input.dispatchMouseEvent', { type: 'mouseReleased', x, y, button: 'left', clickCount: 1 });

/** The middle of the first element `selector` finds, in window points on the half point. */
export const middle = (h, selector) => h.evaluate(`(() => {
  const el = document.querySelector(${JSON.stringify(selector)});
  if (!el) return null;
  const r = el.getBoundingClientRect();
  return [Math.round((r.x + r.width / 2) * 2) / 2, Math.round((r.y + r.height / 2) * 2) / 2];
})()`);

/** `home-team`'s person with more of the account switched on: Multiplayer (so the menu has
 *  their status and Invite people), and what they said, when `said` is handed in. */
export const dressed = ({ multiplayer = false, said = null } = {}) => (snapshot, now) => {
  const out = dressTeam(snapshot, now);
  return {
    ...out,
    alpha: { ...out.alpha, multiplayer },
    team: { ...out.team, me: { ...out.team.me, status: said ? said(now) : null } },
  };
};

/** The pointer onto the middle of `selector` in two steps, and a press there. */
export async function pressOn(h, step, selector, wait = 60) {
  const at = await middle(h, selector);
  if (!at) throw new Error(`nothing on the page answers ${selector}`);
  const [x, y] = at;
  await step('move', x - 8, y - 4);
  await step('move', x, y);
  await step('down', x, y, 0);
  await step('up', x, y, wait);
  return [x, y];
}

/** The pointer onto the middle of `selector` in two steps, resting there. */
export async function restOn(h, step, selector) {
  const at = await middle(h, selector);
  if (!at) throw new Error(`nothing on the page answers ${selector}`);
  const [x, y] = at;
  await step('move', x - 8, y - 4);
  await step('move', x, y);
  return [x, y];
}

/** `dressed`'s person with Rooms (alpha) on and live, the rooms knowing them, none of them in
 *  one yet: the menu offers Go invisible. */
export const withRooms = (dress) => (snapshot, now) => {
  const out = dress(snapshot, now);
  const me = {
    person_id: out.team.me.id, display_name: out.team.me.name, avatar_url: null, headline: null, bio: null,
    show_working: 'team', show_agents: true, show_level: true, show_status: true, invisible_until: null, status_text: null, status_until: null,
  };
  return {
    ...out,
    alpha: { ...out.alpha, rooms: true },
    team: { ...out.team, rooms: { on: true, ready: true, rooms: [], messages: {}, me, ranks: {}, invites: [], mentions: [], error: null } },
  };
};

/** A scene's whole walk: the sidebar opened (or kept open), the row pressed, and `then`. */
export async function openAccount(h, { kept = false, then = null, rested = 400 } = {}) {
  const script = [];
  let t = 0.05;
  // One step for both windows: `move`, `down` and `up` at (x, y), or `type` with the words in x.
  const step = async (type, x, y, wait = 30) => {
    if (type === 'move') await move(h, x, y);
    if (type === 'down') await down(h, x, y);
    if (type === 'up') await up(h, x, y);
    if (type === 'type') await h.call('Input.insertText', { text: x });
    script.push(type === 'type' ? { t: Number(t.toFixed(3)), type, text: x } : { t: Number(t.toFixed(3)), type, x, y });
    t += wait / 1000;
    await h.wait(wait);
  };
  if (kept) {
    await h.evaluate(`localStorage.setItem('agentbox.sidebarShrinks', '0')`);
    await h.goto(h.origin);
    await h.evaluate('(async () => { await document.fonts.ready; })()');
    await h.wait(2000);
  } else {
    // From the page beside it onto Home's row, and resting there until it has opened.
    const [hx, hy] = await middle(h, 'button.workspace-tab[data-tab="home"]');
    await step('move', hx + 80, hy);
    await step('move', hx, hy, 900);
  }
  let pointer = await pressOn(h, step, '.th-me-btn');
  if (then) pointer = (await then(h, step)) ?? pointer;
  // The shutter falls `rested` after the last step's own wait, in both windows.
  return { pointer, rested, settle: rested, script, at: Number((t + rested / 1000).toFixed(3)), ...(kept ? { picks: { rail_shrinks: false } } : {}) };
}
