# What an agent asks the person for: the web window's behaviour, line by line

The checklist of everything the web window puts in front of the person when an agent, or the
app, waits on them inside a conversation, read off its code, and whether this window does the
same. A line is ticked only once it is ported and checked; how it was checked is said on the
line. A line left open says exactly why.

This list exists because one of these was missed: a thread whose agent waited on a sign-in said
Asking at its foot and drew nothing to answer with. Part A is that card, ported. Part B is every
other ask of the same kind a sweep of the web window's conversation code found, none of them
ported yet, so that the next one starts from here and not from memory.

The spec, by file (`R` is `renderer/src`, `S` is `shared`):

- `S/sign-in-ask.mts` (`openSignIn`, `serviceId`, `linkHost`, `linkIsTheService`, `comesBackTo`),
  `R/components/SignInCard.tsx`, where `R/components/ItemThread.tsx` draws it (`signInHere`),
  `R/components/need-card.css` (`.need*`), `R/connections/logos.ts` and its `logos/`.
- The engine's side: `zero:sign-in-answer` and `zero:sign-in-keep` in `main/ipc.mts`.
- For part B: `R/components/NeedCard.tsx`, `GitHubSignInCard.tsx`, `ThreadsMade.tsx`,
  `RemoteControl.tsx`, `CompactionResult.tsx`, `R/ship-offer.ts`, `R/close-asks-first.ts`,
  `R/key-asks.ts`, and their uses in `R/components/Focus.tsx` and `R/App.tsx`.

Each line: **[x]** done, **[ ]** not done. "Tests" are `cargo test` in `desktop/` (the rule's
own cases in `src/rules/sign_in.rs`, the window's in `src/need.rs`, the logos' in
`src/logos.rs`); "pictures" are the scenes `tools/scenes/thread-sign-in*.mjs` photographed from
the web renderer and drawn here from the same fixture.

## A. The sign-in an agent waits on (`SignInCard.tsx`)

### A1. When there is a card

- [x] A1.1 A thread with a `signIn` that has an id, is not `done`, on a thread that is not done,
  and whose `signInAnswer` is not to that id (`openSignIn`). Tests
  `an_ask_is_open_until_it_is_answered_done_or_its_thread_is`,
  `an_answered_ask_and_a_finished_thread_have_no_card`.
- [x] A1.2 Under everything said and what the person did after it, over the line that says
  Asking: 16 points over it and 6 under, which the line's own 14 take (`.need`). In all three
  shapes of the page: a chat being got ready, the newest answer alone, the conversation. Test
  `a_thread_whose_agent_waits_on_a_sign_in_draws_its_card_under_the_conversation`; pictures (the
  card stands at the same height as the web window's in every scene).
- [x] A1.3 Never wider than 640 points, the strong line round it on the raised ground, the
  small shadow, corners of 6. Pictures.
- [x] A1.4 Another ask is another card: nothing typed or said carries over (`key={ask.id}`),
  and leaving the conversation puts the card away with what it held. Test
  `escape_lets_go_of_a_box_and_leaving_the_conversation_starts_the_card_again`.
- [x] A1.5 Not on the phone (`onPhone()`): this window is the desktop's.

### A2. What it says

- [x] A2.1 "Sign in at {host}", the host a browser opens for the link, whatever the agent named
  the service. Tests in `rules/sign_in.rs`.
- [x] A2.2 The mark: the service's own logo when the link is one of the service's own addresses
  and the catalogue has one (23 of them), else a key. A black logo is drawn light (GitHub,
  Intercom, Sentry, Vercel, Expo). Tests in `logos.rs` (every logo draws, the copies are the web
  window's files byte for byte); pictures `thread-sign-in` (key), `-code` (GitHub, light),
  `-paste` (Supabase, in colour), `-cloud` (Vercel, light).
- [x] A2.3 The agent's reason as a sentence: first letter raised, a full stop unless it ends one.
  Wraps. Tests; seen on a real thread whose reason takes two lines.
- [x] A2.4 A code to type on the service's page: "Enter this code", the code in the mono face
  spaced out, Copy. Picture `thread-sign-in-code`.

### A3. What its controls do

- [x] A3.1 "Open {host}" opens the link in the browser (`openArtifactExternally`, which for an
  address is the system's browser). Not while a write is on its way.
- [x] A3.2 "I've signed in" writes `zero:sign-in-answer` with `said: 'signed-in'`; "Not now"
  writes `'skipped'`. Tests
  `ive_signed_in_writes_the_answer_and_the_card_holds_still_until_the_engine_has_it`,
  `not_now_writes_skipped_and_a_refusal_is_said_under_the_buttons`.
- [x] A3.3 While a write is on its way every button stands at half strength and takes no press
  (`busy`); the link "Not now" keeps its look and takes none either. Picture
  `thread-sign-in-busy`; tests.
- [x] A3.4 A write the engine does not take is said under the buttons: its own reason, else
  "That did not reach the agent. Try again." A new press takes the line back. Picture
  `thread-sign-in-refused`; tests, and one for a window with no engine behind it.
- [x] A3.5 A login that waits for a code (`asksForCode`): no "I've signed in", a box "Paste the
  code {service} shows" (the host, when the link is not the service's own) and Send, which takes
  no press over an empty box. Return sends too. What is sent is trimmed (`said: 'code'`). Picture
  `thread-sign-in-paste`; test `a_login_that_waits_for_a_code_takes_it_from_its_box`.
- [x] A3.6 Copy puts the code on the clipboard and says "Copied". Test.
- [x] A3.7 A run on another machine (`session.where === 'cloud'`): the tick "Save this sign-in
  for every cloud thread", on unless the ask says `keep: false`; a press writes
  `zero:sign-in-keep` at once and goes back as it was if the engine does not take it. Picture
  `thread-sign-in-cloud`; test `a_cloud_runs_sign_in_is_saved_for_every_cloud_thread_unless_unticked`.
- [x] A3.8 Such a run with a link that comes back to localhost (`redirect_uri`): "Page did not
  load after signing in?", which gives way to a box "Paste the address from the browser" that
  takes the keys, and Send (`said: 'landed'`). Not on this Mac, where the page loads. Picture
  `thread-sign-in-cloud-pasting`; test.
- [x] A3.9 Keys: a box of the card takes every key typed (E is not Done there); Escape lets go
  of the box and goes no further, and the next Escape is the page's. Tests.
- [x] A3.10 Under the pointer: the main button gives up a little of itself, a quiet button
  takes the wash, a link turns the text's ink. The pointer keeps its arrow over all of them, as
  the web window's does. Pictures `thread-sign-in-hover-open`, `-hover-signed`, `-hover-skip`.
- [x] A3.11 Once the engine has the answer the thread says so in its own fields and the card
  goes with the next reading; a reply written on the thread instead ends the wait the same way
  (`'replied'`, the engine's own doing on a reply).
- [ ] A3.12 Tab does not walk the card's buttons, and Space or Return on one that has the keys
  is not kept from the page (`onKeyDown` on `.need`). No button of this window takes the keys
  from Tab; the boxes do.

### A4. How it was measured

The card is the same size in both windows in every scene (640 by 127, 186.59, 175, 184.59, 205
and 160 points) and stands at the same height on the page. The two windows wear different
palettes at the time of writing (the web window's dark skies went from the warm Ash to a deep
black in an upstream change this window has not followed yet), so the card was compared as ink
over its own ground rather than pixel for pixel: 0.7 to 1.3 of 255.

## B. The other asks, not ported

Each of these leaves the person with a thread that waits on them and nothing, or less, to answer
it with. Worst first.

- [x] B1 **What a cloud run needs** (`NeedCard.tsx`): one card per `AGENTBOX-NEEDS:` line
  (connect a service with its token fields or GitHub's device code, add a key, send a file from
  this computer, enter a one-time code, "Do it on my computer instead"), and their done states.
  Ported: `docs/parity-cloud-needs.md`, which says what of it is still open (E9.2) and lists the
  deviations for the owner.
- [x] B2 **A key or a code typed into the reply box of a thread that asked for one** is saved as
  the key, or sent as the code, and never goes out as words (`keyFromReply` in `key-asks.ts`,
  `Focus.tsx`). Ported: `docs/parity-cloud-needs.md`, part F.
- [x] B3 **"Sign in to GitHub" on a ship the push refused** (`GitHubSignInCard.tsx`), and the
  answer's label "Not shipped". Ported: part C below.
- [x] B4 **The app's own question after a run that left a change and asked nothing**
  (`ship-offer.ts`): "Ship this change?", "This change was never committed.", and the two for a
  Git with no name or files to add, answered by the numbers. Ported: `docs/parity-ship-and-close.md`.
- [x] B5 **What a close asks first** (`close-asks-first.ts`): on unshipped work, "Have the agent
  ship it / Close it anyway / Keep this thread open"; on a thread with filed tasks waiting,
  "Start the task in a new thread, then close this one / Close both / Keep this thread open".
  Ported: `docs/parity-ship-and-close.md`, which says what of it is still open (full screen).
- [x] B6 **The threads this thread filed** (`ThreadsMade.tsx`): "Filed from this thread" or
  "N proposals waiting on you", each with Dismiss, Do here and New thread, and Dismiss all and
  Start all. Ported: `docs/parity-ship-and-close.md`.
- [ ] B7 **"Waiting on {thread} · open →"** over a blocked thread, and the wait banner with
  "stop waiting".
- [ ] B8 **The actions row under a thread put off or scheduled**: "Start it now", "Send it
  now", "Let it run now", "Back to inbox", and when it comes back.
- [ ] B9 **The remote control card** and **the card of a slash command's result**
  (`RemoteControl.tsx`, `CompactionResult.tsx`): this window has no slash commands, so the
  second never has anything to show; the first shows in the web window whenever remote control
  is on, wherever it was turned on.
- [ ] B10 **The card to allow or deny a command** (`App.tsx`, `KeyCardActions.tsx`): left out by
  agreement, as the person runs with full access. Its line says so and asks for Mercury's own
  window.
- [ ] B11 A session started outside Mercury, its choices and "Respond in terminal": this window
  lists no such session (`rules.rs`).

## C. The app's own note, and a ship the push refused

The spec: `R/thread-history.ts` (a `note` line: `forTheAgent`, `says: 'app'`), `S/notes-for-the-agent.mts`,
`R/one-thing-at-a-time.ts` (`appWrote`, `footOf`, `newestAnswer`, `earlierCount`), their use in
`R/components/ItemThread.tsx` (`byApp`, `answer`, `signInWhy`) and `Thread.tsx`
(`e.says === 'app'`), `R/one-thing-at-a-time.css` (`.one-app-note`), `R/styles.css`
(`.appendix-*`), `S/not-shipped.mts`, `S/github-push-sign-in.mts`,
`R/components/GitHubSignInCard.tsx`, and `zero:github-browser-sign-in` and
`zero:github-sign-in` in `main/ipc.mts`. Tests in `src/talk.rs`, `src/rules/shipping.rs` and
`src/need.rs`; scenes `thread-app-note*` and `thread-not-shipped*`.

### C1. A note the app wrote on the row

- [x] C1.1 A note written as `system` is the app's (`says: 'app'`): never signed as the
  agent's, never tagged a checkpoint. A note the agent wrote is still its checkpoint. Tests
  `the_apps_own_note_is_the_apps_…`, `the_agents_own_checkpoint_is_still_an_answer_to_lead_with`.
- [x] C1.2 A note the app wrote for the next run (a cloud machine that stopped, a ship waiting
  for memory, a cause cleared) is drawn by no thread (`forTheAgent`). Tests.
- [x] C1.3 The page that leads with the newest answer leads with the agent's, not with the
  app's note at the foot: the note is one quiet line under the answer, 13 points (or 0.72 of
  the reading size where that is more) in the faint ink, eight points under it, as written and
  not as markdown. "Show N earlier messages" counts all but that answer. Scene `thread-app-note`
  (the line at the web's y, 305.54, and size, 760 by 20.8); tests.
- [x] C1.4 Unfolded, the note at the conversation's foot is that quiet line, as markdown, 14
  points under the conversation's own foot and with no room of its own under it, where an
  agent's last word wears "Result" or "Latest checkpoint". Scene `thread-app-note-unfolded`
  (610.66 in both).
- [x] C1.5 In the middle of a conversation the note is the same quiet line where it was
  written, with no name over it. Scene `thread-app-note-said` (half a point from the web's,
  the page's own rounding).
- [ ] C1.6 The web window signs the note with the app's name for a screen reader (`from`). This
  window gives it no name.

### C2. A ship that failed

- [x] C2.1 While the row's note says the work did not reach main and the agent has not spoken
  since (`notShipped`), the conversation's foot is that note under "Not shipped", an accent dot
  of six points before the words, whoever is taken to have written the note. Scene
  `thread-not-shipped` (label, note and card each within half a point of the web's); tests.
- [x] C2.2 As the thread first opens the page leads with the agent's answer and the note is the
  quiet line under it, with no card: the card is at the conversation's foot. Scene
  `thread-not-shipped-folded`; test.
- [x] C2.3 The card is drawn only for a push that could not sign in to GitHub
  (`asksForGitHubSignIn`): a ship that failed for any other reason went back to its agent.
  Test `a_ship_the_push_refused_ends_on_not_shipped_with_the_card_that_signs_in`.

### C3. The card (`GitHubSignInCard.tsx`)

- [x] C3.1 "Sign in to GitHub", or "Sign in to GitHub again" when GitHub refused the sign-in
  that was saved; GitHub's mark, drawn light. Scenes `thread-not-shipped`, `-again`.
- [x] C3.2 "Sign in with GitHub" starts GitHub's own sign-in (`zero:github-browser-sign-in`);
  while it starts the button says "Starting" and takes no press. Test
  `sign_in_with_github_starts_githubs_own_sign_in_and_shows_its_code`.
- [x] C3.3 Started: the code to enter with Copy ("Copied"), "Open GitHub", which opens the
  address the engine gave (it is opened once by itself as it arrives, as the web window does),
  and "Ships again as soon as you approve it." Scene `thread-not-shipped-device` (640 by 189.59
  in both); test.
- [x] C3.4 It could not start: the card says why (the engine's reason, else "GitHub's sign-in
  did not start.") and opens the token's box by itself. "Paste a token instead" opens it too.
  Scene `thread-not-shipped-token`; tests.
- [x] C3.5 The token's box: the three facts, the first with a link to GitHub's own page for a
  token with the one box a push needs ticked; a box that hides what is typed, "Paste the
  token"; Save, which takes no press over an empty box, says "Checking" while it is checked,
  and writes `zero:github-sign-in` with the token trimmed. Return saves too. Scene
  `thread-not-shipped-again` (the box holding a token); test
  `where_githubs_sign_in_cannot_start_the_card_takes_a_token`.
- [x] C3.6 A token the engine does not take is said under the box (its reason, else "That did
  not save. Try again."); one it takes is said in a line, "GitHub connected. Shipping it
  again." A letter typed takes back what was said. Tests.
- [x] C3.7 Escape lets go of the box; leaving the conversation starts the card again. Test.
- [ ] C3.8 Tab does not walk the card's buttons (as A3.12).

Who wrote an answer where there are several agents ("Claude Code, Opus 5.5" in the label's
place and after "Latest checkpoint") is not here: every label says "Result" or "Latest
checkpoint".
