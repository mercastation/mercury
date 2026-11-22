// The cloud button pressed: its menu open (`CloudButton` in renderer/src/threads/Pages.tsx), on
// a cloud that is on, takes what does not fit on this Mac, has one account and ran a few hours
// this month. On or off, what on means, the accounts, the month's use and the foot line.
//
// The page reads Agentbox Cloud's plan from its server for the month's hours; the shot's bridge
// has no server, so the plan is none and the hours are this Mac's own count (`cloudMonthLine`).
import { CLOUDS, withCloud } from './inbox-cloud.mjs';

export const MONTH = { runs: 12, agentHours: 3.42, estimatedCost: 0, costPerAgentHour: null };

/** Presses the cloud button, as a hand does. */
export async function openCloud(h) {
  await h.evaluate(`document.querySelector('.th-right .th-cloud')?.click()`);
  await h.wait(400);
  return { inboxMenu: 'cloud' };
}

/** The Inbox on `cloud`, with its menu open. */
export async function cloudMenu(h, cloud) {
  const fed = await withCloud(h, cloud);
  return { ...fed, ...(await openCloud(h)) };
}

export const go = (h) => cloudMenu(h, { ...CLOUDS.on, month: MONTH });
