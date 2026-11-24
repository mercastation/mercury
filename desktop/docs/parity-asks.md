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

- [ ] B1 **What a cloud run needs** (`NeedCard.tsx`): one card per `AGENTBOX-NEEDS:` line of the
  newest answer (connect a service with its token fields or GitHub's device code, add a key,
  send a file from this computer, enter a one-time code, "Do it on my computer instead"), and
  their done states. This window draws the marker line as text; `rules/needs.rs` reads the lines
  only for the Inbox's word.
- [ ] B2 **A key or a code typed into the reply box of a thread that asked for one** is saved as
  the key, or sent as the code, and never goes out as words (`keyFromReply` in `key-asks.ts`,
  `Focus.tsx`). Here it is sent as words.
- [ ] B3 **"Sign in to GitHub" on a ship the push refused** (`GitHubSignInCard.tsx`), and the
  answer's label "Not shipped".
- [ ] B4 **The app's own question after a run that left a change and asked nothing**
  (`ship-offer.ts`): "Ship this change?", "This change was never committed.", and the two for a
  Git with no name or files to add, answered by the numbers.
- [ ] B5 **What a close asks first** (`close-asks-first.ts`): on unshipped work, "Have the agent
  ship it / Close it anyway / Keep this thread open"; on a thread with filed tasks waiting,
  "Start the task in a new thread, then close this one / Close both / Keep this thread open".
  Here E closes, or is refused in one line.
- [ ] B6 **The threads this thread filed** (`ThreadsMade.tsx`): "Filed from this thread" or
  "N proposals waiting on you", each with Dismiss, Do here and New thread, and Dismiss all and
  Start all.
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
