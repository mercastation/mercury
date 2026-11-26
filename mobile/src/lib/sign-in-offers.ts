// WHICH OUTSIDE SIGN-INS THIS CLOUD OFFERS. Mercury's self-hosted cloud has
// only email switched on, so a Continue with Apple or Google button there could
// only fail ("provider is not enabled"). Every Supabase says which it offers at
// /auth/v1/settings, to anyone holding the public key. A cloud that cannot be
// asked keeps both buttons, as the app always drew them.
export interface Offers { apple: boolean; google: boolean }

export const BOTH: Offers = Object.freeze({ apple: true, google: true });

/** The offers in a /auth/v1/settings answer; anything else is read as not knowing. */
export function offersOf(settings: unknown): Offers {
  const external = (settings as { external?: unknown } | null)?.external;
  if (!external || typeof external !== 'object') return BOTH;
  const on = external as Record<string, unknown>;
  return { apple: on.apple === true, google: on.google === true };
}

export async function signInOffers(cloud: { url: string; anonKey: string }, fetchImpl: typeof fetch = fetch): Promise<Offers> {
  try {
    const res = await fetchImpl(`${cloud.url.replace(/\/+$/, '')}/auth/v1/settings`, { headers: { apikey: cloud.anonKey } });
    return res.ok ? offersOf(await res.json()) : BOTH;
  } catch {
    return BOTH;
  }
}
