// AN IPHONE BUILD FOR A FREE APPLE ACCOUNT ASKS FOR NOTHING IT CANNOT SIGN
// (w-dcdf1ccfec, 2026-11-25).
//
// The owner installs the iPhone app by cable with a free Apple account rather
// than pay for TestFlight. Xcode's Personal Team cannot sign push or Sign in
// with Apple, and it refuses the whole build when either is asked for. Leaving
// their plugins out of the list was tried first and changed nothing: Expo
// applies an installed expo-apple-authentication's and expo-notifications'
// plugins by itself, and the built entitlements still asked for both. So
// MOBILE_PERSONAL_TEAM=1 adds a plugin of ours (mobile/plugins/), listed first
// so it runs last, that takes the two entitlements back out, and tells the app, which then offers no
// alerts switch that could only fail, no first-run step it never reaches, and
// no Continue with Apple.
import { describe, it, expect, vi, afterEach } from 'vitest';
import fs from 'node:fs';
import { spawnSync } from 'node:child_process';

const mobile = new URL('../mobile/', import.meta.url);
const read = (rel) => fs.readFileSync(new URL(rel, mobile), 'utf8');
const hasExpo = fs.existsSync(new URL('node_modules/expo/config-plugins.js', mobile));
const named = (p) => (Array.isArray(p) ? p[0] : p);
const STRIP = './plugins/without-paid-entitlements';

async function configWith(env) {
  vi.resetModules();
  const was = process.env.MOBILE_PERSONAL_TEAM;
  if (env === undefined) delete process.env.MOBILE_PERSONAL_TEAM;
  else process.env.MOBILE_PERSONAL_TEAM = env;
  try {
    return (await import('../mobile/app.config')).default;
  } finally {
    if (was === undefined) delete process.env.MOBILE_PERSONAL_TEAM;
    else process.env.MOBILE_PERSONAL_TEAM = was;
  }
}

afterEach(() => vi.resetModules());

describe('the build config', () => {
  it('a build for a free account takes the paid entitlements out last, and says so to the app', async () => {
    const config = await configWith('1');
    expect(named(config.plugins[0])).toBe(STRIP);
    expect(fs.existsSync(new URL('plugins/without-paid-entitlements.js', mobile))).toBe(true);
    expect(config.extra.personalTeam).toBe(true);
  });
  it('keeps every other plugin it asks for', async () => {
    const free = (await configWith('1')).plugins.slice(1).map(named);
    expect(free).toEqual((await configWith(undefined)).plugins.map(named));
  });
  it('every other build is as it was', async () => {
    const config = await configWith(undefined);
    expect(config.plugins.map(named)).not.toContain(STRIP);
    expect(config.ios.usesAppleSignIn).toBe(true);
    expect(config.extra).not.toHaveProperty('personalTeam');
  });
  it('only the exact switch turns it on', async () => {
    expect((await configWith('0')).extra).not.toHaveProperty('personalTeam');
    expect((await configWith('')).extra).not.toHaveProperty('personalTeam');
  });
});

// What Xcode is asked to sign, as Expo works it out; needs the app's packages
// (cd mobile && npm ci), which a checkout without them skips.
const entitlements = (env) => {
  const r = spawnSync('npx', ['expo', 'config', '--type', 'introspect', '--json'], { cwd: mobile, encoding: 'utf8', env: { ...process.env, MOBILE_PERSONAL_TEAM: env } });
  if (r.status !== 0) throw new Error(r.stderr);
  return JSON.parse(r.stdout).ios.entitlements ?? {};
};

describe('what Xcode is asked to sign', () => {
  it.skipIf(!hasExpo)('a build for a free account asks for neither push nor Sign in with Apple', () => {
    const asked = entitlements('1');
    expect(asked).not.toHaveProperty('aps-environment');
    expect(asked).not.toHaveProperty('com.apple.developer.applesignin');
  }, 120_000);
  it.skipIf(!hasExpo)('every other build still asks for both', () => {
    const asked = entitlements('');
    expect(asked).toHaveProperty('aps-environment');
    expect(asked).toHaveProperty('com.apple.developer.applesignin');
  }, 120_000);
});

describe('the app built that way', () => {
  it('reads the switch from the copy of the config it carries', () => {
    expect(read('src/platform/personal-team.ts')).toMatch(/Constants\.expoConfig\?\.extra\?\.personalTeam === true/);
  });
  it('offers no alerts, and says why where the switch would be', () => {
    const alerts = read('src/platform/alerts.ts');
    expect(alerts).toMatch(/export const alertsHere = Platform\.OS !== 'web' && !PERSONAL_TEAM;/);
    expect(alerts).toMatch(/free Apple account/);
    expect(read('src/ui/SettingsPages.tsx')).toMatch(/<Row first label="Alerts on this phone" desc=\{alertsWhyNot\} \/>/);
  });
  it('counts the first run in three steps, since the alerts step never comes', () => {
    expect(read('src/ui/Onboarding.tsx')).toMatch(/PERSONAL_TEAM \? SETUP_STEPS\.filter\(\(s\) => s !== 'alerts'\) : SETUP_STEPS/);
  });
  it('draws no Continue with Apple', () => {
    expect(read('src/app/sign-in.tsx')).toMatch(/const showApple = \(apple \|\| c\.preview\) && !!offers\?\.apple && !PERSONAL_TEAM;/);
  });
});
