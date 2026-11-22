// The cloud button pressed on a Mac offered the cloud and never set up: the menu is one
// invitation with one button (`th-cloud-invite`), not choices that would mean nothing yet.
import { CLOUDS } from './inbox-cloud.mjs';
import { cloudMenu } from './inbox-cloud-menu.mjs';

export const go = (h) => cloudMenu(h, CLOUDS.named);
