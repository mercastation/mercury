// WHICH DEMAND A NEW THREAD SERVES, ASKED OF THE SMALL MODEL.
//
// Nobody files a thread under a demand by hand (shared/demands.mts). The app
// asks once per thread, the way it asks for a name (main/row-label.mts): one
// of the project's demands, a new one and its name, or none. The pacing is the
// supervisor's (`placeTheThreads`); what is here is pure.
import { DEMAND_NAME_MAX } from '../shared/demands.mts';

export interface PlaceItem {
  id: string;
  status?: string;
  kind?: string;
  title?: string;
  body?: string;
  label?: string;
  problem?: string;
  labels?: readonly string[];
  parent?: string;
  createdAt?: number;
  updatedAt?: number;
  [field: string]: unknown;
}

/** A thread nobody has touched for this long keeps whatever place it has. */
export const PLACE_WINDOW_MS = 14 * 86_400_000;
/** How long a thread waits for its name before it is placed without one. */
export const PLACE_AFTER_MS = 15 * 60_000;

/** Rows that are not a piece of work toward anything: chats, a teammate's or a repeating task's runs, the tutorial's. */
const NOT_WORK = ['from-chat', 'first-run'];
const NOT_WORK_PREFIX = ['teammate:', 'repeat:'];
const NOT_WORK_KIND = new Set(['question', 'review', 'cause']);

/** PURE. Does this thread still want a place? `placed` is every thread somebody already placed. */
export function wantsPlace(item: PlaceItem, placed: ReadonlySet<string>, now: number = Date.now()): boolean {
  if (placed.has(item.id) || NOT_WORK_KIND.has(String(item.kind))) return false;
  const labels = (item.labels ?? []).map(String);
  if (labels.some((l) => NOT_WORK.includes(l) || NOT_WORK_PREFIX.some((p) => l.startsWith(p)))) return false;
  if (!String(item.title ?? '').trim() && !String(item.body ?? '').trim()) return false;
  // A recorded meeting serves several demands at once; its tasks are each asked about (main/meetings.mts writes this opening).
  if (/^I recorded a meeting\b[^:]*:/.test(String(item.body ?? ''))) return false;
  if (now - (Number(item.updatedAt) || 0) > PLACE_WINDOW_MS) return false;
  return !!String(item.label ?? '').trim() || now - (Number(item.createdAt) || 0) >= PLACE_AFTER_MS;
}

/** PURE. The one message the placing call gets. */
export function placePrompt(item: PlaceItem, demands: readonly { name: string; about?: string }[]): string {
  const cut = (s: unknown, n: number) => String(s ?? '').replace(/\r/g, '').trim().slice(0, n);
  const about = cut(item.problem, 400);
  return [
    'A person keeps their work as demands: a demand is one thing they want delivered, and',
    'threads are the pieces of work that serve it. Here is a thread and the demands its',
    'project already has. Say which demand the thread serves. Reply with one line and',
    'nothing else:',
    '- the number of the demand it serves, when it is part of the same piece of work as',
    '  one of them, even when it is another step or another ticket of it;',
    '- NEW: and a name for a new demand, when it is work toward something none of them',
    '  covers. Name it wide enough that later threads on the same piece of work fit under',
    '  it: the feature or the goal, never the ticket or the step. 2 to 6 words, sentence',
    '  case, in the language the demands below are named in, or with none yet, in the',
    '  language the thread is written in;',
    '- NONE when it is a passing question or housekeeping that delivers nothing of its own.',
    '',
    '--- the thread ---',
    cut(item.label, 100),
    cut(item.title, 300),
    cut(item.body, 800),
    ...(about ? ['--- what it is about ---', about] : []),
    '--- the demands ---',
    ...(demands.length ? demands.map((d, i) => `${i + 1}. ${d.name}${d.about ? `: ${cut(d.about, 200)}` : ''}`) : ['(none yet)']),
    '---',
  ].join('\n');
}

export type Place = { to: 'demand'; index: number } | { to: 'new'; name: string } | { to: 'none' };

/** PURE. What the model answered, or null for a reply that is none of the three. `count` is how many demands it was shown. */
export function readPlace(text: unknown, count: number): Place | null {
  const first = String(text ?? '').split('\n').map((l) => l.trim()).filter(Boolean)[0] ?? '';
  if (/^none\b/i.test(first)) return { to: 'none' };
  const fresh = first.match(/^new\s*:\s*(.*)$/i);
  if (fresh) {
    const name = fresh[1].replace(/^["'`“”‘’]+|["'`“”‘’]+$/g, '').replace(/[.。]+$/, '').trim();
    return name && name.length <= DEMAND_NAME_MAX && name.split(/\s+/).length <= 8 ? { to: 'new', name } : null;
  }
  const n = first.match(/^(\d+)\.?$/);
  if (!n) return null;
  const index = Number(n[1]) - 1;
  return index >= 0 && index < count ? { to: 'demand', index } : null;
}
