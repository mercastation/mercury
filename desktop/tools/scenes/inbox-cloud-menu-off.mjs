// The cloud menu with the cloud off: Off picked, and the line under the choices says where
// every thread runs.
import { CLOUDS } from './inbox-cloud.mjs';
import { MONTH, cloudMenu } from './inbox-cloud-menu.mjs';

export const go = (h) => cloudMenu(h, { ...CLOUDS.off, month: MONTH });
