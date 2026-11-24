// `thread-sign-in` for a login that waits for a code the service's page shows the person
// (`supabase login`): a box to paste it in and Send, and no "I've signed in".
import { signIn } from './thread-sign-in.mjs';

export const SUPABASE = { service: 'Supabase', url: 'https://supabase.com/dashboard/cli/login?session_id=4f1c2a', why: 'to apply the storage rules', asksForCode: true };

export const go = (h) => signIn(h, SUPABASE);
