// The cloud menu after an account was not added: the banner says why, and offers to try again.
import { CLOUDS } from './inbox-cloud.mjs';
import { MONTH, cloudMenu } from './inbox-cloud-menu.mjs';

const failed = { engine: 'codex', key: 'a-codex', state: 'failed', url: null, code: null, message: 'The code had expired.' };

export const go = (h) => cloudMenu(h, { ...CLOUDS.on, month: MONTH, signIns: [failed] });
