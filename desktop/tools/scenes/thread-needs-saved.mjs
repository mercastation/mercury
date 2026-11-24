// `thread-needs` on a Mac whose cloud already keeps the Supabase token: "Supabase connected",
// "Every cloud thread can use it.", and Replace.
import { needs } from './thread-needs.mjs';

export const go = (h) => needs(h, { keys: ['SUPABASE_ACCESS_TOKEN'] });
