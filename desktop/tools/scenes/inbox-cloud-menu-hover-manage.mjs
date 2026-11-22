// The cloud's menu open, the pointer resting on Manage: its words brighten and its edge goes
// faint (`.th-cloud-manage:hover`).
import { CLOUDS } from './inbox-cloud.mjs';
import { MONTH, cloudMenu } from './inbox-cloud-menu.mjs';
import { rest } from './pointer.mjs';

export async function go(h) {
  const fed = await cloudMenu(h, { ...CLOUDS.on, month: MONTH });
  return { ...fed, ...(await rest(h, '.th-cloud-manage')) };
}
