// WHAT A THREAD IS TOLD ABOUT THE DEMAND IT SERVES (shared/demands.mts): the
// demand's name, what it is, and where its other threads stand, with the id to
// leave one a note by. Pure; the supervisor puts it in the brief.
import type { Demand } from '../shared/demands.mts';

export interface Sibling {
  id: string;
  status?: string;
  title?: string;
  label?: string;
  problem?: string;
  progress?: string;
  updatedAt?: number;
  [field: string]: unknown;
}

/** How many of its other threads a brief names, newest first. */
export const SIBLINGS_SHOWN = 8;

const cut = (s: unknown, n: number) => {
  const text = String(s ?? '').replace(/\s+/g, ' ').trim();
  return text.length > n ? `${text.slice(0, n - 1).trimEnd()}…` : text;
};
const lastStep = (progress: unknown) => String(progress ?? '').split('\n').map((l) => l.trim()).filter(Boolean).pop() ?? '';

function siblingLine(t: Sibling): string {
  const about = cut(t.problem, 160);
  const step = cut(lastStep(t.progress), 100);
  const said = [about, step ? `Last step: ${step}` : ''].filter(Boolean).join(' ');
  return `- ${t.id}, ${cut(t.label || t.title, 60)} (${t.status === 'done' ? 'done' : 'open'})${said ? `: ${said}` : ''}`;
}

export function demandBriefBlock(demand: Pick<Demand, 'name' | 'about'> | null | undefined, siblings: readonly Sibling[]): string {
  if (!demand) return '';
  const head = [`## The demand this thread serves: ${demand.name}`, ...(demand.about ? [cut(demand.about, 300)] : [])];
  if (!siblings.length) return [...head, 'No other thread serves it yet.'].join('\n');
  const newest = [...siblings].sort((a, b) => (Number(b.updatedAt) || 0) - (Number(a.updatedAt) || 0));
  const shown = newest.slice(0, SIBLINGS_SHOWN);
  const left = newest.length - shown.length;
  return [
    ...head,
    'Other threads serve the same demand. Where each stands, so you build on their work and do not redo or contradict it:',
    ...shown.map(siblingLine),
    ...(left > 0 ? [`and ${left} older ${left === 1 ? 'one' : 'ones'}.`] : []),
    'When what you find changes what one of them should do, leave it a note with update_work_item and its id.',
  ].join('\n');
}
