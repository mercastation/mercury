// The Demands place: every demand under the tab its state puts it in, and one
// demand's own page. The parts are the inbox's own (the tab bar, a row's
// cells, a thread's title and the list of threads filed from one), so the
// place reads as another page of the same app. The rules are ./demand-rules.
import type { KeyboardEvent } from 'react';
import { ThreadCrumb } from '../threads/Summary';
import { projectSwatch } from '../threads/composer-rules';
import { updatedWords } from '../threads/page-rules';
import '../threads/pages.css';
import './demands.css';
import { demandSentence, deliveredLines, threadsInOrder, wordForPlace, type DemandRow, type DemandTabId, type DemandTabs, type DemandThread } from './demand-rules';
import type { WorkItem } from '../types';

export const DEMAND_TABS: readonly [DemandTabId, string][] = [['today', 'Today'], ['progress', 'In progress'], ['todo', 'To do'], ['delivered', 'Delivered']];

export const demandKey = (d: { product: string; id: string }) => `${d.product}:${d.id}`;

const onEnter = (run: () => void) => (e: KeyboardEvent) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); run(); } };
const Swatch = ({ slug }: { slug: string }) => <i className="dm-sw" style={{ background: projectSwatch(slug) }} aria-hidden="true" />;
const dayWords = (ts: number) => new Date(ts).toLocaleDateString('en-US', { month: 'short', day: 'numeric' });

/** One mark a thread: done, begun, not begun. Past this many the count alone says it. */
const PIPS_MAX = 14;
function Progress({ row }: { row: DemandRow }) {
  if (!row.total) return null;
  return <>
    {row.total <= PIPS_MAX && <span className="dm-pips" aria-hidden="true">
      {Array.from({ length: row.done }, (_, i) => <i key={`d${i}`} />)}
      {Array.from({ length: row.open }, (_, i) => <i key={`o${i}`} className="now" />)}
      {Array.from({ length: row.total - row.done - row.open }, (_, i) => <i key={`l${i}`} className="off" />)}
    </span>}
    <span className="dm-count" aria-label={`${row.done} of ${row.total} threads done`}>{row.done}/{row.total}</span>
  </>;
}

function DemandLine({ row, now, onOpen }: { row: DemandRow; now: number; onOpen: () => void }) {
  const d = row.demand;
  const delivered = row.state === 'delivered';
  const beside = delivered ? d.outcome ?? '' : row.what;
  const under = delivered ? `${dayWords(row.deliveredAt ?? d.updatedAt)} · ${row.what}` : row.what;
  return <div className="row dm-row" role="button" tabIndex={0} data-demand={d.id} onClick={onOpen} onKeyDown={onEnter(onOpen)}>
    <span className="mark" aria-hidden="true" />
    <div className="row-main th-grid">
      <div className="th-cell-title subject dm-title">
        <span className="dm-name">{row.waitsOnYou && <span className="dm-dot" aria-hidden="true" />}{d.name}</span>
        <span className="th-aside dm-on-phone"><Swatch slug={d.product} />{d.productName}</span>
        {beside && <span className="th-aside dm-on-desk dm-beside">{beside}</span>}
        {delivered && d.outcome && <span className="dm-out dm-on-phone">{d.outcome}</span>}
      </div>
      <div className="th-cell-state">{row.word}</div>
      <div className="th-cell-proj th-col-proj"><Swatch slug={d.product} />{d.productName}</div>
      <div className="th-cell-prio th-col-prio dm-threads">{!delivered && <Progress row={row} />}{delivered && row.what}</div>
      <div className="th-cell-when num dm-last">
        <span className="th-when dm-on-desk">{updatedWords(row.deliveredAt ?? d.updatedAt, now)}</span>
        <span className="th-when dm-on-phone dm-what">{under}</span>
        {!delivered && <span className="dm-on-phone dm-progress"><Progress row={row} /></span>}
      </div>
    </div>
  </div>;
}

/** The week in a line, over the delivered demands. */
function WeekLine({ week }: { week: { total: number; byProject: { name: string; count: number }[] } }) {
  if (!week.total) return null;
  return <div className="dm-week"><b>{week.total} {week.total === 1 ? 'thread' : 'threads'} finished</b> this week<br />{week.byProject.map((p) => `${p.name} ${p.count}`).join(' · ')}</div>;
}

export function DemandsList({ tabs, tab, onTab, week, now, onOpen }: {
  tabs: DemandTabs; tab: DemandTabId; onTab: (t: DemandTabId) => void;
  week: { total: number; byProject: { name: string; count: number }[] };
  now: number; onOpen: (row: DemandRow) => void;
}) {
  const rows = tabs[tab];
  return <>
    <div className="th-bar"><div className="tm-tabs">
      {DEMAND_TABS.map(([id, label]) => <button type="button" key={id} className={`tm-tab${id === tab ? ' on' : ''}`} onClick={() => onTab(id)}>{label}{id !== 'delivered' && tabs[id].length > 0 && <b>{tabs[id].length}</b>}</button>)}
    </div></div>
    <div className="list th-table dm-table">
      <div className="row th-head"><span className="mark" aria-hidden="true" /><div className="row-main th-grid">
        <div>Demand</div><div className="th-col-state">Status</div><div className="th-col-proj">Project</div><div className="th-col-prio">Threads</div><div className="num">{tab === 'delivered' ? 'Delivered' : 'Updated'}</div>
      </div></div>
      {tab === 'delivered' && <WeekLine week={week} />}
      {rows.map((row) => <DemandLine key={demandKey(row.demand)} row={row} now={now} onOpen={() => onOpen(row)} />)}
      {rows.length === 0 && <div className="th-empty"><span>{EMPTY[tab]}</span></div>}
    </div>
  </>;
}

const EMPTY: Record<DemandTabId, string> = {
  today: 'Nothing picked for today',
  progress: 'No demand in progress',
  todo: 'No demand waiting to start',
  delivered: 'Nothing delivered yet',
};

function ThreadLine({ thread, onOpen }: { thread: DemandThread; onOpen: (item: WorkItem) => void }) {
  return <div className="made-row made-quiet">
    <button type="button" className="made-open" onClick={() => onOpen(thread.item)}>
      <span className="made-title">{thread.name}</span>
      <span className={`made-state${thread.place === 'needs' || thread.toStart ? ' waiting' : ''}`}>{wordForPlace(thread)}</span>
    </button>
  </div>;
}

export function DemandPage({ row, onOpenThread, onNewThread, onEnd }: {
  row: DemandRow; onOpenThread: (item: WorkItem) => void; onNewThread?: () => void; onEnd?: () => void;
}) {
  const d = row.demand;
  const lines = deliveredLines(row);
  const threads = threadsInOrder(row);
  return <div className="focus-pane dm-pane">
    <div className="focus-scroll">
      <div className="focus">
        <div className="focus-body markdown agent-thread dm-page">
          <div className="one-sum">
            <h2 className="one-title">{d.name}</h2>
            <div className="one-cols one-whole"><div className="one-col"><Swatch slug={d.product} />{d.productName}{d.about ? ` · ${d.about}` : ''}</div></div>
          </div>
          <div className="outcome one-answer">
            <h1 className="one-q">{demandSentence(row)}</h1>
            {row.state === 'delivered' && d.outcome && <p>{d.outcome}</p>}
          </div>
          {lines.length > 0 && <div className="dm-sec">
            <div className="ts-h">{row.state === 'delivered' ? 'Delivered' : 'Delivered so far'}</div>
            <ol className="ts-trail">{lines.map((line, i) => <li key={i} className={`ts-step${i === lines.length - 1 ? ' ts-step-last' : ''}`}>{line}</li>)}</ol>
          </div>}
          {threads.length > 0 && <div className="made">
            <div className="made-label"><span>{threads.length === 1 ? '1 thread' : `${threads.length} threads`}</span></div>
            <div className="made-list">{threads.map((t) => <ThreadLine key={t.item.id} thread={t} onOpen={onOpenThread} />)}</div>
          </div>}
          {onEnd && <button type="button" className="rule-end dm-end" onClick={onEnd}>Remove this demand</button>}
        </div>
      </div>
    </div>
    {onNewThread && <div className="focus-dock"><div className="focus-dock-inner"><div className="dock-card">
      <button type="button" className="dock-pill" onClick={onNewThread}><span className="dock-pill-text">New thread in this demand…</span></button>
    </div></div></div>}
  </div>;
}

/** The top bar over the place: its name, or the open demand's with the way back. */
export function DemandCrumb({ row, onBack, onRename }: { row: DemandRow; onBack: () => void; onRename?: (name: string) => Promise<boolean> }) {
  return <ThreadCrumb from="Demands" name={row.demand.name} onBack={onBack} onRename={onRename} threadKey={demandKey(row.demand)} />;
}

/** Picked for today or not, where an open thread has Done. */
export function TodayButton({ on, onToggle }: { on: boolean; onToggle: () => void }) {
  return <span className="ts-top"><span className="ts-more-wrap">
    <button type="button" className={`ts-done dm-today${on ? ' on' : ''}`} aria-pressed={on} title={on ? 'Picked for today' : 'Pick for today'} onClick={onToggle}>
      <svg viewBox="0 0 16 16" width="14" height="14" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true">{on ? <path d="M3.5 8.5 6.5 11.5 12.5 4.5" /> : <circle cx="8" cy="8" r="4.5" />}</svg>
      <span>Today</span>
    </button>
  </span></span>;
}

/** The place's name in the top bar. On a phone, where there is no sidebar, it is the way between the two. */
export function PlaceTitle({ here, onInbox, onDemands }: { here: 'inbox' | 'demands'; onInbox?: () => void; onDemands?: () => void }) {
  return <h1 className="workspace-title dm-places">
    <button type="button" className={`dm-place${here === 'inbox' ? ' on' : ''}`} aria-current={here === 'inbox' ? 'page' : undefined} onClick={onInbox}>Inbox</button>
    <button type="button" className={`dm-place${here === 'demands' ? ' on' : ''}`} aria-current={here === 'demands' ? 'page' : undefined} onClick={onDemands}>Demands</button>
  </h1>;
}
