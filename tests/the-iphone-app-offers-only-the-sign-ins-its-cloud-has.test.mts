// THE IPHONE APP OFFERS ONLY THE SIGN-INS ITS CLOUD HAS (w-dcdf1ccfec, 2026-11-25).
//
// Mercury's self-hosted cloud has only email sign-in switched on; asked on
// 2026-11-25, its /auth/v1/settings said `external: { email: true, phone: true }`
// and Apple and Google false. The app's sign-in screen still drew Continue with
// Apple and Continue with Google, which could only fail there. It now asks the
// cloud which it offers and draws those alone, and the "or" line only above
// one of them. A cloud that cannot be asked keeps both, as before.
import { describe, it, expect } from 'vitest';
import fs from 'node:fs';
import { BOTH, offersOf, signInOffers } from '../mobile/src/lib/sign-in-offers';

const OLYMPUS = { external: { anonymous_users: false, apple: false, azure: false, email: true, google: false, phone: true }, disable_signup: true };
const AGENTBOX = { external: { apple: true, email: true, google: true } };

const answering = (status, body, seen = []) => async (url, init) => {
  seen.push({ url, init });
  return { ok: status >= 200 && status < 300, status, json: async () => body };
};

describe('what the cloud says', () => {
  it('a cloud with email alone offers neither', () => {
    expect(offersOf(OLYMPUS)).toEqual({ apple: false, google: false });
  });
  it('a cloud with both offers both', () => {
    expect(offersOf(AGENTBOX)).toEqual({ apple: true, google: true });
  });
  it('one switched on is that one', () => {
    expect(offersOf({ external: { google: true } })).toEqual({ apple: false, google: true });
  });
  it('an answer with no list is read as not knowing', () => {
    expect(offersOf(null)).toEqual(BOTH);
    expect(offersOf({})).toEqual(BOTH);
    expect(offersOf({ external: 'yes' })).toEqual(BOTH);
  });
});

describe('asking it', () => {
  it('asks the auth settings with the public key, at the address without its slash', async () => {
    const seen = [];
    const got = await signInOffers({ url: 'https://api.example.com/', anonKey: 'public-key' }, answering(200, OLYMPUS, seen));
    expect(got).toEqual({ apple: false, google: false });
    expect(seen[0].url).toBe('https://api.example.com/auth/v1/settings');
    expect(seen[0].init.headers).toEqual({ apikey: 'public-key' });
  });
  it('a refusal keeps both', async () => {
    expect(await signInOffers({ url: 'https://api.example.com', anonKey: 'k' }, answering(503, null))).toEqual(BOTH);
  });
  it('no network keeps both', async () => {
    expect(await signInOffers({ url: 'https://api.example.com', anonKey: 'k' }, async () => { throw new Error('offline'); })).toEqual(BOTH);
  });
});

describe('the sign-in screen', () => {
  const signIn = fs.readFileSync(new URL('../mobile/src/app/sign-in.tsx', import.meta.url), 'utf8');
  it('draws each outside sign-in only when offered, and the "or" line only above one', () => {
    expect(signIn).toMatch(/\{\(showApple \|\| showGoogle\) && <View/);
    expect(signIn).toMatch(/\{showApple && <Btn label=\{busy === 'apple'/);
    expect(signIn).toMatch(/\{showGoogle && <Btn label=\{busy === 'google'/);
    expect(signIn).toMatch(/const showGoogle = !!offers\?\.google;/);
  });
  it('asks before the early return, so the hooks keep their order', () => {
    expect(signIn.indexOf('signInOffers(cfg)')).toBeLessThan(signIn.indexOf('if (c.signedIn) return <Redirect'));
  });
});
