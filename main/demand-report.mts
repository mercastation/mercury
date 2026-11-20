// WHAT A DEMAND'S THREADS DELIVERED, WRITTEN BY THE SMALL MODEL.
//
// Each thread already keeps a Context and a Done list. A demand's report is
// those, read together: what the demand is, what has been delivered so far
// and, once every thread is done, the outcome. Written onto the demand
// (shared/demands.mts `about`, `delivered`, `outcome`), never onto a thread.
// The pacing is the supervisor's (`reportTheDemands`); what is here is pure.
import type { Demand } from '../shared/demands.mts';

export interface ReportThread {
  id: string;
  status?: string;
  title?: string;
  label?: string;
  problem?: string;
  progress?: string;
  result?: string;
  updatedAt?: number;
  [field: string]: unknown;
}

/** A demand is not written about more often than this, however busy its threads are. */
export const REPORT_EVERY_MS = 10 * 60_000;

const said = (t: ReportThread) => [t.problem, t.progress, t.result].some((s) => String(s ?? '').trim());

/** PURE. Is a report due: has a thread of it said something since the last one? */
export function wantsReport(demand: Pick<Demand, 'wroteAt'>, threads: readonly ReportThread[], now: number = Date.now()): boolean {
  const spoke = threads.filter(said);
  if (!spoke.length) return false;
  const wroteAt = demand.wroteAt ?? 0;
  if (now - wroteAt < REPORT_EVERY_MS) return false;
  return Math.max(...spoke.map((t) => Number(t.updatedAt) || 0)) > wroteAt;
}

/** PURE. The one message the report call gets. */
export function reportPrompt(demand: Pick<Demand, 'name'>, threads: readonly ReportThread[]): string {
  const cut = (s: unknown, n: number) => String(s ?? '').replace(/\s+/g, ' ').trim().slice(0, n);
  const steps = (s: unknown) => String(s ?? '').split('\n').map((l) => l.trim()).filter(Boolean).join('; ').slice(0, 500);
  const allDone = threads.length > 0 && threads.every((t) => t.status === 'done');
  return [
    'A person keeps their work as demands: a demand is one thing they want delivered, and',
    'threads are the pieces of work that serve it. Below is a demand and what its threads',
    'say they did. Write its report in the language the threads are written in, for',
    'someone busy who will not open the threads:',
    'ABOUT: one plain sentence saying what the demand is.',
    'DELIVERED: up to five lines, one per thing actually delivered so far, each a phrase',
    'of under fifteen words that says the outcome and not the steps. Keep a ticket id or',
    'pull request number that identifies it. No line for work that is still going.',
    ...(allDone ? ['OUTCOME: one or two sentences saying what was delivered in the end.'] : []),
    'Reply in exactly this shape and with nothing else:',
    'ABOUT: …',
    'DELIVERED:',
    '- …',
    ...(allDone ? ['OUTCOME: …'] : []),
    '',
    '--- the demand ---',
    cut(demand.name, 100),
    '--- its threads ---',
    ...threads.slice(-12).flatMap((t) => [
      `[${t.status === 'done' ? 'done' : 'open'}] ${cut(t.label || t.title, 100)}`,
      ...(cut(t.problem, 1) ? [`Context: ${cut(t.problem, 400)}`] : []),
      ...(steps(t.progress) ? [`Done: ${steps(t.progress)}`] : !cut(t.problem, 1) && cut(t.result, 1) ? [`Said: ${cut(t.result, 300)}`] : []),
    ]),
    '---',
  ].join('\n');
}

export interface Report { about?: string; delivered: string[]; outcome?: string }

const SENTENCES_MAX = 320;
const LINE_MAX = 140;

/** The whole sentences of a text that fit, so nothing ends half said. */
function wholeSentences(text: string, max: number): string {
  if (text.length <= max) return text;
  const fits = text.slice(0, max + 1);
  const end = Math.max(fits.lastIndexOf('. '), fits.lastIndexOf('! '), fits.lastIndexOf('? '));
  return end > 0 ? fits.slice(0, end + 1) : `${text.slice(0, max - 1).replace(/\s+\S*$/, '')}…`;
}

/** PURE. The report in a reply, or null for a reply with none of its parts. */
export function readReport(text: unknown): Report | null {
  const lines = String(text ?? '').split('\n').map((l) => l.trim());
  const after = (head: string) => lines.find((l) => l.toUpperCase().startsWith(`${head}:`))?.slice(head.length + 1).trim() || undefined;
  const from = lines.findIndex((l) => /^DELIVERED\s*:/i.test(l));
  if (from < 0 && !after('ABOUT')) return null;
  const delivered: string[] = [];
  for (const l of from < 0 ? [] : lines.slice(from + 1)) {
    if (/^(OUTCOME|ABOUT)\s*:/i.test(l)) break;
    const line = l.replace(/^[-•*]\s*/, '').trim();
    if (line && line !== '…') delivered.push(wholeSentences(line, LINE_MAX));
  }
  const about = after('ABOUT');
  const outcome = after('OUTCOME');
  return {
    ...(about ? { about: wholeSentences(about, SENTENCES_MAX) } : {}),
    delivered: delivered.slice(0, 5),
    ...(outcome ? { outcome: wholeSentences(outcome, SENTENCES_MAX) } : {}),
  };
}
