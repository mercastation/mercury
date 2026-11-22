// The cloud menu while an account has stopped working: the banner says so and offers Fix. The
// button behind it still says Sign in again; it stops once the menu closes.
import { CLOUDS } from './inbox-cloud.mjs';
import { MONTH, cloudMenu } from './inbox-cloud-menu.mjs';

export const go = (h) => cloudMenu(h, { ...CLOUDS.again, month: MONTH });
