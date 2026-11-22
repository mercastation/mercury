// The cloud menu while an account is being added in the background: the banner says so, with
// the moving mark of a wait and nothing to press.
import { CLOUDS } from './inbox-cloud.mjs';
import { MONTH, cloudMenu } from './inbox-cloud-menu.mjs';

export const go = (h) => cloudMenu(h, { ...CLOUDS.adding, month: MONTH });
