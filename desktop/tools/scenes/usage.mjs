// Readings of the agents' limits, invented, and the pointer brought onto the rail's usage row
// so its card opens (`UsagePill` in renderer/src/components/UsagePill.tsx).
//
// Each reading is in the engine's own shape (`Usage` in renderer/src/types.ts): Claude Code
// prints a clock time for each reset, with the day when it is not today; Codex hands over an
// instant and no words, which the card counts down. The zone is left out, so both windows read
// "today" on this machine's own calendar, as they do for a reading taken here.
const MIN = 60_000;
const HOUR = 60 * MIN;
const DAYS = ['Sun', 'Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat'];

/** "6:10pm", "7am": a clock time as Claude Code prints it beside a reset. */
const clock = (at) => {
  const d = new Date(at);
  const h = d.getHours() % 12 || 12;
  return `${h}${d.getMinutes() ? `:${String(d.getMinutes()).padStart(2, '0')}` : ''}${d.getHours() < 12 ? 'am' : 'pm'}`;
};

/** A reset at a whole minute this far ahead, half a minute clear of the next one. */
const ahead = (now, ms) => Math.floor((now + ms) / MIN) * MIN + MIN / 2;

export function claude(now, { read = 30_000 } = {}) {
  const session = ahead(now, 2 * HOUR + 10 * MIN);
  const week = ahead(now, 3 * 24 * HOUR + 5 * HOUR);
  const limit = (span, qualifier, name, percent, at) => ({
    span, qualifier, name, percent, resetsAt: at, resetsText: clock(at), resetsOn: DAYS[new Date(at).getDay()], zone: null,
  });
  return {
    engine: 'claude', at: now - read, profile: 'default',
    limits: [
      limit('session', null, 'This session', 4, session),
      limit('week', 'all models', 'This week', 39, week),
      limit('week', 'Opus', 'This week, Opus', 45, week),
    ],
  };
}

export function codex(now, { read = 26 * HOUR } = {}) {
  return {
    engine: 'codex', at: now - read, profile: 'default',
    limits: [
      { span: 'week', qualifier: null, name: 'This week', percent: 12, resetsAt: ahead(now, 6 * 24 * HOUR + 14 * HOUR), resetsText: null, resetsOn: null, zone: null },
    ],
  };
}

/** The invented inbox, with these readings on it and both agents offered on this Mac. */
export const dressed = (readings) => (snapshot, now) => {
  const all = readings(now);
  return {
    ...snapshot,
    usage: all[0],
    usageByEngine: all,
    engines: { workspace: 'claude', choices: [{ id: 'claude', label: 'Claude Code' }, { id: 'codex', label: 'Codex' }], byItem: {} },
  };
};

/** The middle of the rail's usage row, in window points. */
const row = (h) => h.evaluate(`(() => {
  const r = document.querySelector('.workspace-usage .usage-pill').getBoundingClientRect();
  return [Math.round((r.x + r.width / 2) * 2) / 2, Math.round((r.y + r.height / 2) * 2) / 2];
})()`);

const move = (h, x, y) => h.call('Input.dispatchMouseEvent', { type: 'mouseMoved', x, y });

// Folded, the sidebar draws no bar (`.workspace-collapsed … .workspace-usage` in sidebar.css),
// so the pointer goes onto the folded sidebar's last row first, in two steps, as a hand moves:
// the sidebar opens 140 ms later. Once it is open the bar stands at the end of that row, and the
// pointer goes along the row onto it, again in two steps; the card comes 120 ms after that.
// `script` is the same pointer for the native window (`--script`), and `rested` how long after
// its last move the shutter falls.
export async function pointAt(h, { rested = 400 } = {}) {
  const [fx, fy] = await row(h);
  await move(h, fx - 8, fy - 4);
  await h.wait(30);
  await move(h, fx, fy);
  await h.wait(600);
  const [x, y] = await row(h);
  await move(h, x - 8, y - 4);
  await h.wait(30);
  await move(h, x, y);
  const script = [
    { t: 0.05, type: 'move', x: fx - 8, y: fy - 4 },
    { t: 0.08, type: 'move', x: fx, y: fy },
    { t: 0.68, type: 'move', x: x - 8, y: y - 4 },
    { t: 0.71, type: 'move', x, y },
  ];
  return { pointer: [x, y], rested, settle: rested, script, at: 0.71 + rested / 1000 };
}

// The same with the rail kept open (Settings' "the sidebar shrinks to icons" turned off): the
// card comes 120 ms after the pointer moves onto the row.
export async function pointAtOpen(h, { rested = 400 } = {}) {
  await h.evaluate(`localStorage.setItem('agentbox.sidebarShrinks', '0')`);
  await h.goto(h.origin);
  await h.evaluate('(async () => { await document.fonts.ready; })()');
  await h.wait(2000);
  const [x, y] = await row(h);
  await move(h, x - 8, y - 4);
  await h.wait(30);
  await move(h, x, y);
  const script = [
    { t: 0.05, type: 'move', x: x - 8, y: y - 4 },
    { t: 0.08, type: 'move', x, y },
  ];
  return { pointer: [x, y], rested, settle: rested, script, at: 0.08 + rested / 1000, picks: { rail_shrinks: false } };
}
