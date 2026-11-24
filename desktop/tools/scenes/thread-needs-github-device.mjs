// `thread-needs-github` after "Sign in with GitHub": the code to enter with Copy, and "Open
// GitHub", while the approval is waited on (`token` never answers here).
import { needs, press } from './thread-needs.mjs';
import { GITHUB } from './thread-needs-github.mjs';

const DEVICE = { code: 'A1B2-C3D4', url: 'https://github.com/login/device' };

export const go = (h) => needs(h, {
  said: GITHUB,
  bridge: {
    gitHubCloudSignIn: `async (p) => (p?.action === 'start' ? ${JSON.stringify({ ok: true, ...DEVICE })} : new Promise(() => {}))`,
    openArtifactExternally: 'async () => ({ ok: true })',
  },
  then: async (h) => {
    await press(h, 0, 'Sign in');
    await h.wait(300);
    return { cloudNeeds: { cards: { 0: { device: DEVICE } } } };
  },
});
