// The cloud menu while two threads run on this Mac that the cloud could take now: a line for
// them, with the button that moves them (`cloudMoveLine`).
import { CLOUDS } from './inbox-cloud.mjs';
import { MONTH, cloudMenu } from './inbox-cloud-menu.mjs';

export const go = (h) => cloudMenu(h, { ...CLOUDS.on, month: MONTH, movable: ['w-readme0', 'w-readme1'] });
