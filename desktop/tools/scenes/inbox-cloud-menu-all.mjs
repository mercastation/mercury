// The cloud menu with Everything picked, three threads running there and one on its way: the
// line says it is moving, with nothing to press, and the foot counts what runs there.
import { CLOUDS } from './inbox-cloud.mjs';
import { MONTH, cloudMenu } from './inbox-cloud-menu.mjs';

export const go = (h) => cloudMenu(h, { ...CLOUDS.count, month: MONTH, movable: ['w-readme2'] });
