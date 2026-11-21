# How the web window derives Home, the Inbox, In progress, Later and Done

A spec for one person with no team, read off Mercury `cb4a455ca`, so the native window shows the
same rows in the same order. `R` = `renderer/src`, `S` = `shared`. `now` is the clock at render.
The port is `desktop/src/rules.rs` and `desktop/src/rules/`, one Rust function per function named
here; `tools/rows.mjs` holds its lists against the rows the web window draws.

Pointers name a file and a function, not a line: the checkout moved 291 commits while this was
being ported, and every line number in the first draft was stale by the end of it.

Four things that are easy to get wrong:

- The "Urgent" lift is computed and then re-sorted away, and the Inbox page draws no section headings.
- A work item's title goes through `shortTitle`, not `firstRealLine`.
- The Inbox table row has no preview line and no engine letter. Home has the letter and no Status word.
- "No team" is not "nobody signed in". Signed in with Multiplayer off there is still no team
  (`teamView`, `R/team/people.tsx`), but every line of the ledger carries its writer (`wrote[f].by`),
  and one rule below reads that.

## 0. What the page keeps for itself (not in the snapshot)

Renderer state in `R/App.tsx`, from localStorage:

- `deliveredThrough`: `zero.deliveredThrough`, stamped at first launch.
- `unreadSince`: `zero.unreadSince`, same.
- `seenAt{id→ms}`: `zero.seenAt`. `seen` set: `zero.seen` (an imported conversation stays on top until opened).
- `snoozes{id→ms}`: legacy `zero.snooze`, read and never written.
- `troubleClosed`: `zero.troubleClosed`, the `since` of the trouble row the person dismissed.
- `cloudBrokenSeen`: `zero.cloudBrokenSeen`, the stopped cloud accounts already looked at in the
  cloud menu, as `engine:key` (section 9, the cloud button).
- `pendingIds` / `startingIds`: undo-window sets; empty here.
- Assumed: no onboarding walk, `scope = null`, display = `DEFAULT_DISPLAY.inbox` (a list, sorted by
  priority, no filter; `R/threads/page-rules.ts`), box filter off, search closed.

`desktop/src/data.rs` `Local` carries all six. `tools/reference.mjs` writes the first three and the
cloud's into a fixture; a fresh profile has the others empty. The running app's local door hands
over the first three only.

## 1. Data read

**Snapshot** (`R/types.ts`): `items`, `products` (`slug`, `name`, `practice?`), `approvals?` (`{id, item}`),
`supervisor`, `engines?: {workspace, byItem}` (the engine a resting row would run on),
`cloud?: {setUp, keys[{name}]}`, `config.outsideAgents?` (default `'off'`; otherwise terminal-agent rows
join the inbox and In progress, `agentRows` in `App.tsx`), `teammates?`, `team?`, `alpha?`.

**WorkItem**: `id`, `product` (slug), `productName`, `status: 'open'|'claimed'|'done'|'blocked'`,
`kind` (`'question'`, `'review'`, `'cause'`…), `labels[]`, `title`, `label?` (the namer's short name),
`body?`, `answer?` (`'(withdrawn)'` = none), `result?`, `note?`, `priority` (0 = untagged),
`place?` (dragged score), `pinned?`, `placedAt?`, `parent?`, `blockedBy?[]`, `start?: 'later'|'now'`,
`runAt?`, `snoozedUntil?`, `answeredThrough?`, `createdAt`, `updatedAt`, `epoch?`,
`wrote?: Record<field, {ts, source: 'founder'|'agent'|'system', by?, teammate?}>`,
`claim`, `claimExpired?`, `engine?`, `signIn?`, `signInAnswer?`, `waitEnded?: {said, at}`,
`handedOver?: {said, at}`, `shippedAt?`, `linkComments?`, `waitingComments?`, `routed?`,
`runner?`, `createdBy?`, `assignee?`.

**supervisor**: `running: {itemId, product, startedAt, engine?, where?}[]`, `starting?[]`, `stalled?[]`,
`shipping?[]`, `queued?[]`, `runNow?[]`, `remoteHeld?[]`, `signInNeeded?: Record<id, tool>`,
`atLimit?`, `cannotStart?: Record<id, {why, tool}>`, `silent?`, `whyNotCloud?`, `paused?`,
`productOrder?[]`, `spawnTrouble?`. Not read by these lists: `hiddenProducts`, `scheduled`, `capacity`.

## 2. Shared predicates

```
liveAnswer(i)   = i.answer && i.answer != '(withdrawn)'                         R/list-rules.ts
hers(i)         = labels.includes('founder') && wrote.labels?.source != 'agent' S/answers.mts filedByThePerson
QR(i)           = kind=='question' || kind=='review'
stillAsksYou    = QR && status!='done' && !liveAnswer                           list-rules
answerSettled   = liveAnswer && (t=wrote.answer?.ts??0)>0 && (answeredThrough??0)>=t
                  && any f in [result,title,body]: wrote[f].source=='agent' && wrote[f].ts>t   answers
herLastWord     = max wrote[f].ts, f in [title,body,answer], source=='founder'
                  (skip answer when answer=='(withdrawn)')                      list-rules
answeredHerAsk  = status=='open' && hers && !liveAnswer && result
                  && wrote.result?.source=='agent' && wrote.result.ts>herLastWord
gaveUp          = status=='open' && liveAnswer && !answerSettled && wrote.result?.source=='agent'
                  && wrote.result.ts>(wrote.answer?.ts??0) && saysARunGotNowhere(result)
wakesItselfAt   = (status=='open' && !QR && wrote.runAt && wrote.runAt.source!='founder'
                  && runAt>(wrote.result?.ts??0)) ? runAt : 0                   answers
runsAgain       = (!hers || !result?.trim() || offerIsLive(i)) ? 0 : wakesItselfAt     list-rules runsAgainOnItsOwn
putAway(i,legacy)= max(mine, reminder, legacy)                                  list-rules putAwayByThem
   mine     = wrote.runAt?.source=='founder' && !((wrote.answer?.ts??0)>(wrote.runAt?.ts??0)) ? runAt??0 : 0
   reminder = wrote.snoozedUntil?.source=='founder'
              && !(liveAnswer && (wrote.answer?.ts??0)>(wrote.snoozedUntil?.ts??0)) ? snoozedUntil??0 : 0
hiddenAt(i)     = max(putAway(i, snoozes[id]??0), runsAgain(i))                 list-rules hiddenUntil, App hiddenAt
parkedByAgent   = status!='done' && (runAt??0)>now && wrote.runAt?.source!='founder'
setByPerson     = status!='done' && blockedBy?.length
                  && (wrote.blockedBy?.source=='founder' || onOthers)           S/thread-waits.mts
waitingOn(i)    = waitsThatCanEnd(i, byId)   // recursive; ported as is         list-rules
waitsInLater    = !labels.includes('given-task') && (setByPerson
                  || (waitingOn.length>0 && status=='blocked' && wrote.status?.source!='founder'))
waitCameBack    = waitEnded && status!='done' && waitEnded.at>herLastWord
isProposal      = status=='open' && !hers && !liveAnswer && !QR && !!parent
isCleanRun      = !routed && labels.includes('clean') && (a 'repeat:<id>' label, or a label that
                  startsWith 'teammate:' | =='teammate-offer' | =='teammate-asks-to-write')   S/repeats.mts
isCause         = kind=='cause' && labels.includes('cause') && wrote.kind?.source!='agent'  S/causes.mts
wordWaits       = liveAnswer && wrote.answer && wrote.runAt
                  && (wrote.answer.source=='founder' || wrote.answer.teammate)
                  && wrote.answer.ts>wrote.runAt.ts && (answeredThrough??0)<wrote.answer.ts  answers
dueAt(i)        = wordWaits ? 0 : runAt??0                                      list-rules deferredUntil
liveIds         = running[].itemId ∪ starting                                   list-rules agentOnOrComing
askingIds       = approvals[].item (non-null) ∪ {i.id : openSignIn(i)}          App askingIds
   openSignIn   = signIn?.id && !signIn.done && status!='done' && signInAnswer?.id != signIn.id  S/sign-in-ask.mts
stalledIds      = stalled ∪ keys(signInNeeded)                                  R/fleet.ts stoppedIds
shippingIds     = shipping
owed            = threadsOwedAnAnswer(items, now)   // parents of proposals older than 24 h  list-rules
```

Ported verbatim, not summarised here: `saysARunGotNowhere` (`S/spawn-trouble.mts`, with its three tables
of openers), `offerIsLive` and `parseOptions` (`R/format.ts`), `onOthers` (`S/thread-waits.mts`),
`asksToClose` (`S/asks-to-close.mts`).

Everything above keys by `id` alone, as the web window does. Two projects holding a thread of one id
share its answer.

## 3. Inbox membership (`inboxCandidates` in `App.tsx` → `belongsInInbox` in `list-rules.ts`)

```
inInbox(i):
  if wordsWait(i)                                        → yes   // linkComments.length; a teammate's comments
  live=liveIds.has(id); ask=askingIds.has(id)
  if live && !stillAsksYou && !ask                       → no    // being run right now
  if shippingIds.has(id) && !stillAsksYou && !ask        → no
  if owed.has(id)                                        → yes
  if ask                                                 → yes
  // belongsInInbox, in this order:
  if start=='later'                                      → no
  if hiddenAt(i) > now                                   → no    // snoozed / scheduled by the person
  if stalledIds.has(id)                                  → yes
  if isCleanRun                                          → no
  if waitsInLater(i, waitingOn(i))                       → no
  if waitCameBack                                        → yes
  if runsAgain(i)                                        → no
  if parkedByAgent                                       → yes
  if status∉{done,claimed} && answerSettled              → yes
  if answeredHerAsk || gaveUp                            → yes
  if status=='done':
     if isCause                                          → no
     if !hers && !liveAnswer                             → no
     if wrote.status?.source=='founder'                  → no    // closed by the person
     if asksToClose(liveAnswer) && (wrote.status?.ts??0) >= (wrote.answer?.ts??Infinity) → no
     return (wrote.status?.ts ?? updatedAt ?? 0) > deliveredThrough
  if status=='blocked'                                   → yes
  if QR                                                  → !liveAnswer
  if isProposal                                          → no
  if status=='open' && !hers                             → !liveAnswer
  → no
```

Then `answersInTheirChat` (`R/team/answers-in-the-chat.ts`): a task labelled `asked-in:<product>/<id>`
whose chat is among the items leaves the inbox and its chat is listed instead, unless the task is
asking or stalled.

Team hooks in `inboxCandidates`: a thread out with someone asked for help (`askedOut`), file news,
`closedOverWords`, `closedByTheOther`, `teamInbox`.

## 4. Inbox order

**Step 1, `inbox`** (`App.tsx`)

```
score(i) = isFinite(i.place) ? i.place
         : placeScore(productOrder, i.product) + itemPriority(i)                S/rank.mts
   placeScore   = idx<0 ? 0 : (order.length-idx)*100
   itemPriority = isFinite(priority) && priority>0 ? priority : 5
rows  = inInbox items sorted by score desc, then (placedAt ?? updatedAt) desc   list-rules byRunningOrder
fresh = rows where labels.includes('codex') && the first 'codex:<id>' label has an id
        && status!='done' && !seen.has(id)                                      R/import-row.ts justImported
urgent = rest where itemPriority >= 9                                           R/urgent-arrival.ts
inbox = [troubleRow?] + fresh + urgent + ordinary
```

`troubleRow` is a synthetic item with id `'trouble'` and product `''`, present when
`supervisor.spawnTrouble` is set and its `since` is not the one dismissed (`R/trouble-row.ts`). Its
title is `strandedTitle(count)`, its line `troubleSummary`, both its times `since`.

**Step 2, what is drawn** (`sorted` in `page-rules.ts`; Home's `needsList` and the page's `displayedBox`)

```
drawn = rows with product=='' (inbox order) + rest stable-sorted by:
  1. (pinned && status!='done') desc
  2. rankScore desc
       rankScore = isFinite(place) ? place : placeScore(productOrder, product) + bucket
       bucket    = 9|7|5|2 = value of priorityIdOf(itemPriority(i))
  3. fromAPerson desc            // team; 0 alone
  4. (placedAt ?? updatedAt) desc
  5. tie → step-1 order (stable sort)
```

**Sections.** The Inbox page is a table whenever search is closed: one unlabelled group, no headings
and no section counts. Table head literals: `Thread`, `Status`, `Project`, `Priority`, then `Sent`
(`Done` on the Done tab; `timeHeading`).

## 5. In progress and Done

**Membership** (`progress` in `App.tsx` → `belongsInProgress`)

```
inProgress(i):
  live = liveIds.has(id)
  if hiddenAt(i) > now && !live                → no
  if start=='later'                            → no
  if live || shippingIds.has(id)               → yes
  if dueAt(i) > now                            → no
  if stalledIds.has(id)                        → no
  if status=='claimed'                         → yes
  if setByPerson || answerSettled              → no
  if runsAgain(i)                              → yes
  if answeredHerAsk || gaveUp                  → no
  if status=='open' && liveAnswer              → yes
  → status=='open' && !QR && hers
```

**Order.** `progress` = those items by `score` desc, then `placedAt ?? updatedAt` desc (raw priority,
not bucketed). `progressTab` = `progress` minus ids in `inbox` (`inProgressTab`). Home draws
`progressTab.slice(0, 60)` in that order. The page's In progress tab re-sorts: pinned first, then
`sentAt` desc, ties in Home's order (`order` in `page-rules.ts`).

**Home's progress row** (`homeProgress` in `App.tsx`): `letter` = `engineLetter(engine)`, `title` =
`rowTitle(i)`, `line` = `${engineName(engine)} · ${i.productName}`, time = `updatedWords(sentAt(i))`.

**Done tab** (`done` in `App.tsx`): items with `status == 'done'` that are not in the inbox (by id),
not live and not shipping, sorted by `updatedAt` desc, then re-sorted by the page: `finishedAt` desc,
where `finishedAt = wrote.status?.ts || updatedAt`. Its time cell is `updatedWords(finishedAt)`, no
row is unread, and no pin is drawn.

**Later** (`snoozed` in `App.tsx`; the tab's name in the code is still `snoozed`)

```
inLater(i):
  if start=='later' && teammateIdOf(i)                   → no    // an AI teammate's queued run (queuedTeammateRun)
  if status=='done' && wrote.status?.source=='founder'   → no    // closed by the person
  → start=='later' || isProposal(i) || putOffInLater(i)
putOffInLater = (hiddenAt(i) > now || waitsInLater(i, waitingOn(i)))
                && !(liveIds.has(id) || queued.includes(id) || shippingIds.has(id))     list-rules
whenShown(i)  = start=='later' || waitsInLater(i, waitingOn(i)) ? Number.MAX_SAFE_INTEGER : hiddenAt(i)
snoozed       = inLater items, stable-sorted by whenShown ascending
drawn         = sorted(snoozed, 'snoozed')     // step 2 of section 4: pinned, rankScore, placedAt
```

- The page's sort is the Inbox's, so `whenShown` only orders rows that tie on all of it. A proposal
  is not named in `whenShown` and has no `hiddenAt`, so among ties it comes first, not with the rest
  that wait on a person as the comment over it says.
- A thread put off with a run still queued for it is in no tab: not Later (`runOnIt`), not In
  progress (`hiddenAt > now`), not the inbox.
- A thread its agent finished while `start=='later'` is in Later and in Done at once.
- Not ported: the two other kinds of row the web window lists here. Repeating tasks are rules on a
  channel of their own (`zero.repeats`); they are drawn above the threads and counted in the tab's
  number (`snoozed.length + repeats.length`). A terminal agent put off (`agentList`) joins the list
  when the setting lists such agents at all.

`tabState` (inbox wins, then In progress, Later, Done) says where each thread sits, and a thread an
agent filed under another counts toward that thread's "N tasks to approve" only while it sits in
Needs you or Later (`waitingCounts`, `approvableFiled` in `R/threads-made.ts`).

## 6. Home's Needs-you row (`homeNeeds` in `App.tsx`, `R/home/home-needs.ts`, `R/home/HomePage.tsx`)

First 80 entries of `drawn`.

```
letter   = engineLetter(rowEngine(i))                                           R/home/home-model.ts
  rowEngine: run = running.find(itemId==id && product==i.product); if run?.engine → it
     else if i.engine=='codex' || (i.engine!='antigravity'
             && labels.some(l ∈ {'codex','codex-import','not-imported'} || l.startsWith('codex:'))) → 'codex'
     else snap.engines?.byItem?.[id] ?? snap.engines?.workspace ?? i.engine ?? 'claude'
  engineLetter: codex→'X', antigravity→'A', else 'C'
  engineLabel:  'Claude Code' | 'Codex' | 'Antigravity' (unknown → 'Claude Code')
title    = rowTitle(i) = shownTitle(i)                                          list-rules, S/thread-title.mts
  label = (i.label ?? '').trim()
  if !label → shortTitle(title)
  if wrote.title?.source=='founder' && wrote.label && wrote.title.ts > wrote.label.ts && title.trim() → shortTitle(title)
  else wrote.label?.source=='founder' ? label : shortTitle(label)
  // no name at all and a 'teammate:<id>' label → the first line of the result that says something
priority = rowPriorityId(itemPriority(i))                                       page-rules
  priorityIdOf(n): n>=9 'urgent'; n>=7 'high'; n<=2 'low'; else 'medium'        R/priority.ts
  rowPriorityId: 'medium' → null
project  = i.productName
at       = sentAt(i)                                                            page-rules
  no wrote → updatedAt
  by(f) = wrote[f] && wrote[f].source!='agent' ? wrote[f].ts : 0
  max(by('body'), by('answer'), start=='now' ? by('start') : 0) || createdAt || updatedAt
unread   = (wrote.result?.ts ?? createdAt ?? 0) > (seenAt[id] ?? unreadSince)   R/unread.ts
```

- `shortTitle` is pure and about 100 lines of regex tables, ported verbatim. Its word tables are
  ASCII: a word is looked up without its accented letters (`são` as `so`), and `\b` ends a word at an
  accent. The port keeps both, or Portuguese titles cut differently.
- An engine the workspace names (`engines.workspace`) outranks the row's own `engine` field, so a row
  that asked for Antigravity reads `C` while the workspace's answer is Claude Code.
- Bars (`R/components/Priority.tsx`): urgent = 4 bars (`prio-bars p4`); high, medium, low = 3 bars
  with class `p3`, `p2`, `p1` (3, 2, 1 filled). Home draws the mark only when `priority` is non-null.
- Tags: priority mark, then `project` if non-empty. Time is `updatedWords(at)`, the Inbox's words:
  Home said `agoShort` until `921a97caf`, and `agoShort` is now the command bar's only.
- Empty-state literals: `Nothing needs you.` and `No agent is working right now.`
- A run of an AI teammate (`snap.teammates`) wears the teammate's face and name instead of the letter.
  Not ported.

## 7. The Inbox page's row (`ThreadCells` and `RowCells` in `R/threads/Pages.tsx`, rows in `R/components/List.tsx`)

- **Thread:** `rowTitle(i)`. A faint aside follows unless it repeats the title (`asideBesides`):
  the first of a link-comment line (`linkCommentLine`), `waitEnded.said` if `waitCameBack`,
  `handedOver.said`, `${n} task(s) to approve`. Then `not started` if `start == 'later'`, and a pin
  mark if `pinned && status != 'done'`.
- **Status:** start from `statusCell(i)` (`R/status-word.ts`, on `liveLine` in `R/live-line.ts` and
  `threadState` in `S/thread-cards.mts`). Then, on the inbox tab and when no agent is on it:
  link comments make it `Waiting on you`, whatever it was; else a cloud run's unmet asks make it
  `Needs Vercel` / `Needs a key` / `Needs a file` / `Needs a code` / `Needs N things`
  (`needsWord(needsLeft(i, savedKeys))`, `S/cloud-needs.mts`). Else `cellUnderTab` (`page-rules.ts`):
  on the inbox tab `Waiting` becomes `Waiting on you` if `asksForYourWords`, otherwise `Answered`; on
  the progress tab `In progress` draws nothing. Every other word passes through: `Asking`, `Working`,
  `Working locally`, `Moving to cloud`, `Stopped`, `Signed out`, `Nothing came back`, `Not shipped`,
  `Shipped`, `Shipping`, `Queued`, `Up next`, `Not started`, `At its limit`, `Paused`,
  `Nothing running`, `Getting ready`, `Assigning`, `Asks you who`, `Scheduled`, `Done`.
  `Shipping`, `Queued`, `Up next`, `At its limit`, `Paused`, `Nothing running` and the `Not started`
  of a queued row that cannot start are only said under In progress (`Not started` is also the word
  of a thread written down in Later, under any tab). A fleet that was never read
  (`supervisor.queued` absent) says only the state.
- **The mark before the word** (`StatusMark`): none on a word that is the thread's own state; the
  turning half-dot or the cloud while an agent is on it; a ring for a word about an agent that is
  waiting its turn (`Queued`, `Up next`, `Shipping`, `Assigning`) or held (the rest); the solid dot
  when link comments replaced a word that had a mark.
- **A cause row** (`isCause`, not done): Status is `${n} waiting`, counting the blocked threads whose
  `blockedBy` names it; Project is where most of them are, then `+N` for the other projects
  (`causePlace`, `S/causes.mts`); no aside.
- **Project:** `products.find(slug == i.product)?.name ?? ''` (Home uses `i.productName`).
- **Priority:** the same `rowPriorityId(itemPriority(i))`; mark plus label `Urgent` / `High` / `Low`; empty at Medium.
- **Sent:** `updatedWords(sentAt(i), now)`; on Done, of `finishedAt(i)`.
- **Under Later** three things change (`tab === 'snoozed'` in `ThreadCells`):
  - Status starts from `laterCell(i)`, which is `statusCell` with the state fixed as `scheduled`:
    `Scheduled`, or `Not started` when `start == 'later'`, even on a thread whose status is `done`.
    The words ahead of the state in `statusCell` are still said: `Getting ready`, `Assigning`,
    `Asks you who`, `Signed out`, `Stopped`. No tab drops a word here (`cellUnderTab`).
  - The time column is headed `Back` and says when the thread comes back:
    `backWords(i) = dayAndTime(max(hiddenUntil(i), runAt ?? 0))`, or nothing when that is 0 (written
    down and never started, a proposal, a wait on another thread). `hiddenUntil` is asked without
    the legacy `zero.snooze`, so a thread put off only there has an empty cell.
  - A thread behind an open cause (`causeOf`, `S/causes.mts`: `status == 'blocked'` and a
    `blockedBy` naming a cause row that is not done) is a row of its own shape: Status `Waiting`
    with the mark the cell had, the aside is the cause's `title` as filed, the time cell says
    `When fixed`, and it wears no pin and no `not started`.
- A column no row fills is not drawn, heading included; a table 915 points wide or less gives up
  Priority, and one 785 or less gives up Project and narrows Status to 160 (`@container th`).
- A time's words are set to the right of a 96 point column. Wider words (a dated Back) start at the
  column's left and run out past its right, through the row's padding and under the card's edge.
- The title cell is a line of text unless the row has an aside. Too long for the cell, the line is
  cut from its end: the pin goes, then `not started`, then the title's own words. With an aside the
  cell is a row of boxes: the aside takes what the title leaves and is cut first, and `not started`
  and the pin keep their room while the title gives way.
- **Unread:** row class `unread` on the inbox tab only, same rule as Home.
- A row whose label is `codex-import` (a Codex conversation asking to come in) is not drawn as a
  table row at all: it keeps a title, a line and two keys. Not ported.
- The demand a thread serves follows its title (`th-demand`). Demands are not in the snapshot; they
  come on their own channel (`zero.demands`). Not ported.

**The line under a title** (`rowSummary` in `list-rules.ts`) is drawn only where the list is not a
table (search), and is `Row::line`:

```
trouble row            → the first line of its body
notShippedLine(i)      → it                                                     S/not-shipped.mts
i.answer && wrote.answer.by && wrote.answer.ts >= max(wrote.body.ts, wrote.result.ts)
                       → firstRealLine(answer)      // every line has a writer once someone is signed in
result && (status=='done' || wrote.result.ts > wrote.body.ts) → previewText(result)
else                   → previewText(body after its title)
all through clipToSentence (112 characters, whole sentences)
```

## 8. Time strings

```
updatedWords (Home and the Inbox table, page-rules.ts):
  min<1 'just now'; min<60 `${min} min ago`;
  ts>=startOfLocalDay(now) → `${h} hour${h==1?'':'s'} ago`;
  ts>=startOfLocalDay(now)-86400000 → 'Yesterday';
  else Intl {month:'short', day:'numeric'} in the system locale ("Sep 28")
agoShort (the command bar, R/home/home-model.ts):
  m=max(0,floor((now-at)/60000)); m<1 'now'; m<60 `${m} min`; h=floor(m/60); h<24 `${h} hr`; else `${floor(h/24)} d`
dayAndTime (Later's Back column, R/format.ts):
  days = whole local days from today to ts's day
  0 <= days < 7 → Intl {weekday:'short', hour:'numeric', minute:'2-digit'}                 "Sun 5:24 PM"
  else          → Intl {weekday:'short', month:'short', day:'numeric', hour, minute}       "Mon, Nov 30, 5:24 PM"
```

The port writes the dates in English, with a twelve-hour clock. The reference Chrome on this Mac
does too.

## 9. Counts

`shownCount(rows)` = rows passing the display filters; with the default display it is `rows.length`.

| Place | Value | Drawn |
|---|---|---|
| Home's "Inbox" door | `inbox.length` | always, 0 included; lit if > 0 |
| "Needs you" card head | `needsList.length`, uncapped (the list shows 80) | only if > 0 |
| "In progress" door and card head | `progressTab.length` | door always; head only if > 0 |
| Rail's Inbox dot | `inbox.length` | only if > 0 |
| The page's Needs you, In progress and Later tabs | `inbox.length`, `progressTab.length`, `snoozed.length` (+ repeating tasks) | only if > 0 |
| The page's Done tab | none | a word alone (`COUNTED` in `Pages.tsx`) |

**The tab strip** (`StateTabsRow`): Needs you, In progress, Later, Done. Later is drawn only while
`laterTabShown`: its count is over 0, or it is the open tab, or the walkthrough is on
(`R/later-tab.ts`). An empty tab's one line is `emptyTabWords`.

**The cloud button** (`HeaderActions` and `CloudButton` in `Pages.tsx`), between New thread and the
display button, from the snapshot's `cloud`:

```
drawn       = cloud?.setUp || cloud?.offered
count       = cloud.running > 0 ? cloud.running : null                     R/cloud-menu.ts cloudButtonCount
on          = cloud.mode != 'mac'                                          cloudIsOn
nobodyReady = !cloud.accounts?.length && !cloud.signIns?.length
setup       = any signIn 'failed' || any signIn 'checking' || any account 'broken'     R/cloud-setup.ts menuSetupLine
asks        = !nobodyReady && accounts 'broken', as engine:key, not all in cloudBrokenSeen     buttonAsksSignIn
lit         = on && !nobodyReady            // the cloud, and the count, in the accent colour
inside      = asks ? 'Sign in again' : count ? count : nobodyReady ? 'Cloud' : nothing
dot         = asks || (!count && (nobodyReady || setup))
```

A bare square of 32 points; with a count 9 points, the cloud, 7, the number, 10; with a word 9, the
cloud, 7, the word, 14, so the dot has room in the corner. New thread stands 10 points to its left.

## 10. Held against the web window

`node tools/rows.mjs <dir> --shoot <checkout>` photographs five scenes (Home and the page's four
tabs) with `tools/reference.mjs`, reads the rows out of the dumps and compares them with
`mercury --fixture … --rows`, field by field. Under each tab it also holds the strip of tabs with
their numbers, the time column's heading and the cloud button (`--rows --tab <name>` says them). `--cases` feeds both the invented threads in
`tools/rows.cases.mjs`, one for each branch above; `--real` the person's real threads; `--real --stir`
the same with a busy fleet invented around them.

`reference.mjs`'s own `MERCURY_REAL=1` finds no thread on this Mac: the harness it borrows reads
`accounts/<account>/<project>/work-items.jsonl`, and the ledgers are now in the store's `projects`
folder. `rows.mjs` reads them from there and hands the harness the snapshot.
