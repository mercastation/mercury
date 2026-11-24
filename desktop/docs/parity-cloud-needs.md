# What a cloud run needs, and a key typed into the reply box: the web window's behaviour, line by line

The checklist of everything the web window does with the marked lines a cloud run ends its
answer on (`AGENTBOX-NEEDS: …`, and the older `AGENTBOX-NEEDS-A-KEY: …`), read off its code top
to bottom, and whether this window does the same. It is parts B1 and B2 of `parity-asks.md`. A
line is ticked only once it is ported and checked; how it was checked is said on the line. A line
left open says exactly why.

The spec, by file (`R` is `renderer/src`, `S` is `shared`):

- `S/cloud-needs.mts` (`NEED`, `NEED_LINE`, `SERVICES`, `serviceById`, `tidy`, `needsIn`,
  `isNeedLine`, `needId`, `keysOf`, `spokenSince`, `needsLeft`, `needsWord`, `listed`,
  `connectedReply`, `lateReply`, `threadsKeysFree`).
- `S/cloud-keys.mts` (`KEY_ASK`, `ASK_LINE`, `keyAsks`, `isKeyAsk`, `keyWords`, `keyLabel`,
  `savedReply`), `S/cloud-key-sources.mts` (`sourceHoldsSignIns`, `keySource`,
  `keySourceFromWords`, `keySourceLine`, `keySourceWords`), and what they read:
  `S/sign-in-places.mts` (`pathHoldsSignIns`, `keychainItemHoldsSignIns`) and
  `S/cloud-key-names.mts` (`engineReads`).
- `R/key-asks.ts` (`NEED_BLOCK`, `needsAsBlocks`, `KEY_BLOCK`, `keyAsksAsBlocks`, `keyBlock`,
  `keySavedWords`, `keyShaped`, `keyFromReply`, `isNewestAnswer`, `SHAPES`, `keyLooksWrong`,
  `BEGINS`, `codeLooksWrong`).
- `R/components/NeedCard.tsx` (`NeedPlace`, `HELD`/`setHeld`, `NeedOlder`, the icons, `Mark`,
  `LINK`/`withLinks`, `lower`, `sentence`, `giveBackKeys`, `focusNextAsk`, `keepKeys`,
  `sizeWords`, `fileWhy`, `fileName`, `folderOf`, `saysOnlyTheTitle`, `placeName`, `placeItem`,
  `FromHere`, `KeyFields`, `GitHubOwnSignIn`, `Footer`, `NeedCard`), `R/components/need-card.css`.
- `R/components/Focus.tsx`: `needName`, the block "WHAT A CLOUD RUN ASKED FOR" (`keysJustSaved`,
  `savedKeys`, `latestNeeds`, `latestAsks`, `heldHere`, `keyedOpen`, `openAfter`, `heldWords`,
  `saveKeys`, `saveLate`, `sendFile`, `sendCode`, `sendReply`, `gitHubSignIn`, `md` with its
  `live`), the `pre` of `mdComponents` that turns a `agentbox-need` block into a card, and
  `DockComposer`'s `send` (what text reaches `sendReply`, `clearDraft` before it).
- `R/components/Thread.tsx` (`asking`, `NeedOlder`), `R/components/ItemThread.tsx` and
  `R/components/OneThingAtATime.tsx` (`QueueAnswer`: the newest answer alone), and `R/App.tsx`
  (`cloudKeys` from `snap.cloud.setUp` and `snap.cloud.keys`, `onNotice` is `showToast`,
  `onReplySend` is `answerWith`, `onReplyClose` folds the box).
- The engine's side, `main/ipc.mts`: `zero:cloud-key`, `zero:github-cloud-sign-in`,
  `zero:cloud-file-info`, `zero:cloud-send-file` (its `choose` opens the Electron window's own
  panel), `zero:cloud-send-code`; `shared/bridge-map.mts` for the bridge's names.
- The web repo's tests: `a-cloud-run-asks-for-what-only-the-person-can-give`,
  `a-cloud-run-missing-a-key-asks-for-it-on-the-thread`, `a-cloud-ask-is-answered-once-and-checked`,
  `an-answered-ask-can-still-be-given`, `an-ask-with-a-command-in-it-still-draws-its-card`,
  `a-cloud-github-ask-signs-in-with-githubs-own-login`,
  `the-key-cards-say-where-on-this-computer-the-key-comes-from`,
  `a-key-card-never-reads-a-sign-in-the-cloud-may-not-have`,
  `a-cloud-run-gets-a-key-from-this-computer-only-after-a-yes` (its `keySource*` cases).

Each line: **[x]** done, **[ ]** not done. "Tests" are `cargo test` in `desktop/`: the rules'
own cases in `src/rules/needs.rs` and `src/rules/key_asks.rs` (each web test's cases, named after
it in a comment), the window's in `src/cloud_need.rs`. "Pictures" are the scenes
`tools/scenes/thread-needs*.mjs`, photographed from the web renderer and drawn here from the same
fixture; each card's rectangle is compared with the web's (section H).

## D. The rules

### D1. Reading the lines (`S/cloud-needs.mts`, `S/cloud-keys.mts`)

- [x] D1.1 `needsIn`: each `AGENTBOX-NEEDS:` line of the four kinds, in order and once each by its
  id, the closing backtick taken off only when the line opened with one, the reason `tidy`d (a
  leading dash, colon or space off); then every older `AGENTBOX-NEEDS-A-KEY:` line. Tests
  `each_kind_of_line_is_read_with_what_it_is_for`,
  `an_ask_whose_reason_holds_inline_code_is_read_code_and_all`.
- [x] D1.2 A sign-in names one of the twelve services, by id, any case; any other is left out.
  Each service's name, its variables with their labels, and its token page (`SERVICES`). Tests
  (a capital service, a service off the list), `the_services_each_have_a_page_to_make_a_token_on`.
- [x] D1.3 A key's name is capital letters, digits and underscores, two to eighty; what it is is
  the reason, else the words after the name; a third part after a second bar that reads as a
  place is where on this computer it is, and is taken off what it is; a third part that is not a
  place, or names one that holds sign-ins, stays in what it is. Test
  `a_place_after_the_key_is_read_as_a_third_part`.
- [x] D1.4 A file is a full path (`~/` or `/`); a code is any word. Tests.
- [x] D1.5 `keyAsks` and `isKeyAsk`: the older line, backticks round it taken off, the name and
  what it is, once per name. Test `the_older_line_is_read_with_its_name_and_what_it_is`.
- [x] D1.6 `isNeedLine`, `needId`, `keysOf`, `needsLeft`, `needsWord`; the Inbox's row still reads
  `needs_word`. Tests `a_row_says_what_its_cloud_run_waits_on`,
  `a_reply_since_or_a_closed_thread_waits_on_nothing`,
  `what_a_thread_still_waits_on_leaves_out_what_is_saved`,
  `a_need_saves_every_variable_of_a_sign_in_and_none_for_a_file_or_a_code`.
- [x] D1.7 `keyWords` and `keyLabel`. Test `a_key_is_named_without_what_comes_after`.
- [x] D1.8 `connectedReply`, `lateReply`, `savedReply`, `threadsKeysFree`, `listed`. Tests
  `a_saved_sign_in_frees_every_open_thread_waiting_on_it`,
  `a_key_that_comes_late_is_said_to_have_been_saved_after_all`,
  `saving_says_the_same_words_the_thread_does`. (`threadsKeysFree`, `savedReply`, `isNeedLine`
  and `NEED` are read by no screen of this window: the engine frees the threads; kept with their
  cases as the web keeps them.)
- [x] D1.9 The need as the web window's block holds it (`JSON.stringify(need)`, its fields in the
  same order). Test `a_need_is_its_block_as_the_web_window_writes_it`.

### D2. Where a key is on this computer (`S/cloud-key-sources.mts`)

- [x] D2.1 `keySource`, `sourceHoldsSignIns`, `pathHoldsSignIns` (this copy's own home is
  `~/.mercury`), `keychainItemHoldsSignIns`, `engineReads`. Test
  `a_key_is_read_only_from_a_place_that_holds_no_sign_in` (every case of the two web tests).
- [x] D2.2 `keySourceFromWords`, `keySourceLine`, `keySourceWords`. The same test.

### D3. What the window checks (`R/key-asks.ts`)

- [x] D3.1 `needsAsBlocks`. Tests `a_need_is_its_block_as_the_web_window_writes_it`,
  `an_ask_whose_reason_holds_inline_code_is_read_code_and_all`.
- [x] D3.2 `keyAsksAsBlocks` and `keyBlock` (drawn by no screen of the web window any more:
  `Focus.tsx` reads only `NEED_BLOCK`, and `needsIn` reads the older line). Test
  `the_older_line_is_a_block_the_thread_reads_back`.
- [x] D3.3 `keySavedWords`, every sentence. Test `what_saving_says`.
- [x] D3.4 `keyFromReply`. Tests `a_key_pasted_into_the_reply_box_is_the_one_key_asked_for`,
  `a_reply_that_says_where_the_key_is_goes_as_words`.
- [x] D3.5 `isNewestAnswer`. Test `the_ask_that_takes_a_key_is_the_newest_answer`.
- [x] D3.6 `keyLooksWrong` (with `SHAPES` and `BEGINS`). Test
  `a_key_pasted_under_the_wrong_name_is_warned_about`.
- [x] D3.7 `codeLooksWrong`. Test `a_code_that_is_not_six_digits_is_asked_about_twice`.

## E. The cards (`NeedCard.tsx`)

### E1. When and where a card is drawn

- [x] E1.1 Each need line of an agent's message is a card where the line stood, the words above
  and below it as markdown, on the page that leads with the newest answer, in the conversation's
  messages, and in the last word at its foot (`cloud_need::blocks`, called where
  `thread.rs` drew `markdown::blocks` for an agent's words). Test
  `a_line_a_cloud_run_ends_on_is_a_card_where_it_stood`; every scene, `thread-needs-unfolded`.
- [x] E1.2 No cloud set up: the line is the block's own source, a code block of the need as JSON.
  Test `on_a_mac_with_no_cloud_the_line_is_the_block_the_web_window_falls_back_on`; picture
  `thread-needs-no-cloud` (the same block, cut at the column's edge as the web's is).
- [x] E1.3 `.need`: 16 over and 6 under, 10 between two cards, never wider than 640, the strong
  line on the raised ground, the small shadow, corners of 6 (`need::framed`). Pictures: every card
  stands at the web's height, the second and later of a message too (`-several`, `-two`,
  `-answered`).
- [x] E1.4 Which card takes anything (`live`): the newest answer as drawn, an open thread,
  nothing said since (a reply taken back is not said), and in a conversation the agent's last
  word with nothing of the person's after it (`NeedOlder`, `asking`). Tests
  `an_ask_answered_in_words_says_so_and_can_still_be_given`; pictures `-answered`, `-unfolded`.
- [x] E1.5 Leaving the conversation lets go of what every card held; each card is its own, by
  its message and its place among the message's cards. Test
  `escape_lets_go_of_a_box_and_leaving_the_conversation_lets_go_of_the_cards`.
- [x] E1.6 `showFacts`, `showFoot`, `keyedLeft`, `othersLeft` over the card's own message. Tests
  (the way out under the last open, facts on the first); pictures `-two` ("Kept the same way."),
  `-several`, `-held`.

### E2. The mark (`Mark`)

- [x] E2.1 A sign-in wears its service's logo (a dark one drawn light), a code the logo of what
  it is for when there is one, else a key; a file the file (`need::mark_of`, the kit's mark made
  general). Pictures (Supabase, Stripe, Vercel, GitHub, Sentry, npm, the key, the file). AWS's
  letters, which its file sets as text in the system's bold sans, are set over its smile by this
  window in the same face (`mark_of`; picture `-aws`, test
  `awss_mark_has_its_letters_over_its_smile`).

### E3. Connect a service, add a key (`NeedCard`, `KeyFields`)

- [x] E3.1 "Connect {service}", "Add the {keyWords(what) or name}". Tests; pictures.
- [x] E3.2 The reason as a sentence with its links (opened in the browser) and its code on its
  chip; a key's `what` unless it only says the title. Tests; pictures `thread-needs-key`,
  `-key-here`.
- [x] E3.3 The facts: "Get {a|an} {plain} from {Service}'s own page" (link), "Kept encrypted in
  your Mercury Cloud", "Every thread can use it from now on"; past the first, "Get … own page. Kept
  the same way.". Pictures `thread-needs`, `-two`, `-several`; tests.
- [x] E3.4 One box per variable, hiding what is typed, "Paste the {plain}", "Paste the {label}"
  for several, "Or paste it" under a press that reads the key; Save after the last, a quiet one
  under that press. Pictures `-aws`, `-key-here`, `-warned`; tests.
- [x] E3.5 Save held over empty boxes and while saving ("Saving"); "Paste the {label} too.";
  a warning, then "Save anyway" until a box changes, and the second press saves; Return the same;
  typing takes back what was said. Tests
  `a_value_that_looks_wrong_is_warned_about_and_the_second_press_saves_it_anyway`,
  `a_sign_in_of_two_variables_asks_for_both_and_saves_each_in_order`,
  `a_refused_save_is_said_under_the_boxes_and_a_letter_typed_takes_it_back`; pictures `-aws`,
  `-warned`, `-saving`.
- [x] E3.6 Saved: nothing said on the card, the next open card's box takes the keys; refused: the
  reason, else "The cloud did not take the key. Try again.". Tests (and `focusNextAsk` asserted in
  `a_file_sent_while_a_sign_in_is_open_waits_for_it_and_goes_with_its_save`); picture `-refused`.
- [x] E3.7 "Do it on my computer instead" under the last open card, laptop and arrow in the faint
  ink, the words the text's ink under the pointer; sends "Do it from my computer." as any reply,
  leaving the reply box as it was. Test
  `the_way_out_sends_do_it_from_my_computer_as_any_reply_and_keeps_the_box`; pictures (the foot is
  the web's 638 by 41.8 in every scene that has it). The pointer's look on it is not photographed.

### E4. A key where the run said it is (`FromHere`)

- [x] E4.1 "Use the one in your Keychain / your shell / {file name}", and beside it `$NAME`,
  "ENTRY in ~/path" or the path, mono, cut short, its whole as the tooltip. Test
  `a_key_where_the_run_said_it_is_is_read_and_saved_in_one_press`; picture `-key-here`.
- [x] E4.2 A press saves with an empty value and `from`, "Reading" meanwhile; a refusal said under
  it. The same test. Nothing of this window reads the Keychain, a shell or a file: the engine
  does, on `zero:cloud-key`.

### E5. GitHub's own sign-in (`GitHubOwnSignIn`)

- [x] E5.1 "Sign in with GitHub" (arrow), "Paste a token instead", no box, no token page;
  "Starting". Test `githubs_own_sign_in_saves_the_token_gh_made`; picture `-github`.
- [x] E5.2 The code with Copy (which says nothing, as `writeText` does), "Open GitHub"; GitHub's
  page opened once as the code arrives. Same test; picture `-github-device`.
- [x] E5.3 Then the approval is waited on (`token`, given up to fifteen minutes) and the token
  saved as `GH_TOKEN`, "GitHub token"; a start, an approval or a save that fails takes the code
  away and says why ("GitHub's sign-in did not start.", "… did not finish."). Same test.
- [x] E5.4 "Paste a token instead": the facts with GitHub's token page, and the box. Same test;
  picture `-github-token`.

### E6. A sign-in or key that is saved, or answered

- [x] E6.1 "{service} connected" / "{label} saved", "Every cloud thread can use it.", Replace on
  the live card (the boxes again, empty). The way out the web window draws under a saved card is
  never drawn there either (`showFoot` cannot be true for a done card). Test
  `a_token_pasted_is_saved_for_every_cloud_thread_and_the_card_says_connected`; picture `-saved`.
- [x] E6.2 Not live and not saved: "Answered below.", "Connect it now" / "Add it now", which opens
  it without the way out, its first box taking the keys, and saves as a late key. Test
  `an_ask_answered_in_words_says_so_and_can_still_be_given`; pictures `-answered`, `-late`.

### E7. Send a file

- [x] E7.1 "Send {name}", where it is and how big, or that it is not there, then the reason; the
  size asked as the card comes up (`zero:cloud-file-info`). Test
  `a_file_is_sent_from_this_computer_or_one_picked_in_this_windows_own_panel`; pictures `-file`,
  `-file-missing`, `-several`. Asked once a visit: see "Deviations" 4.
- [x] E7.2 "Goes only to this thread's cloud machine, and is deleted when it ends". Pictures.
- [x] E7.3 "Send from this computer" and "Choose another file"; only "Choose the file" for a file
  that is not there; "Sending", both held. Test; pictures.
- [x] E7.4 Choosing shows this window's own file panel ("Pick the file to send") and sends the
  path picked; closing it sends nothing and frees the card. Never the web window's panel. Test.
  See "Deviations" 3.
- [x] E7.5 Held while another ask is open: "{name} is ready", "Ready. It goes with …" (or
  "Sent."). Test `a_file_sent_while_a_sign_in_is_open_waits_for_it_and_goes_with_its_save`;
  picture `-held`.
- [x] E7.6 "Send it now"; sent late: "{name} sent", "This thread has it now.". Test; pictures
  `-answered`, `-sent-late`.
- [x] E7.7 Refused: the reason, else "The file could not be sent. Try again.". Test.

### E8. Enter a code

- [x] E8.1 "Enter the {about} code", the reason "Goes to this thread only.", "Enter it once {X}
  is in: these codes run out in a minute." while a sign-in or key is open. Test
  `a_code_waits_for_the_sign_ins_and_is_checked_before_it_goes`; pictures `-code`, `-held`.
- [x] E8.2 Its box: "123456", 16 points spaced 0.18 em, 150 wide; no spaces, twelve at most;
  Send ("Sending") held over an empty box; Return sends. Same test; picture `-code`. See
  "Deviations" 5.
- [x] E8.3 "Connect {X} first, then the code: it runs out in a minute."; the six-digit warning,
  then sent anyway. Same test; picture `-code`.
- [x] E8.4 Refused: the reason, else "The code could not be sent. Try again.". Same test.
- [x] E8.5 Not live: "Enter the {about} code", "Answered below.", nothing to press. Test;
  picture `-answered`.
- [x] E8.6 The way out under it when it is the last open. Picture `-code`.

### E9. Keys

- [x] E9.1 Escape in a box lets go of it and goes no further (`need::in_a_box`). Test
  `escape_lets_go_of_a_box_and_leaving_the_conversation_lets_go_of_the_cards`.
- [ ] E9.2 Enter and Space on a card's button (`keepKeys`) and a box selecting what it holds as it
  takes the keys (`select()` on focus). No button of this window takes the keys (as A3.12 of
  `parity-asks.md`), so the first has nothing to keep; the second is not ported: Rune's field
  has no hook for gaining the keys, and a press in a box puts the caret where it lands.

## F. The reply box of a thread that asked (`sendReply`)

- [x] F1 Only with the cloud set up; read off `item.result` whatever was said since, as the web
  window reads it. Tests in `cloud_need.rs` (`a_code_typed_into_the_reply_box_goes_as_the_code`).
- [x] F2 A code of 4 to 8 digits: sent as the code with what was held, the box folded and emptied,
  never words; while a sign-in or key is open, "Connect {X} first, then the code: it runs out in a
  minute." and nothing sent. Test `a_code_typed_into_the_reply_box_goes_as_the_code`.
- [x] F3 A key: saved as that key, the box folded and emptied with its draft, its pick and its
  files, `keySavedWords` said; inside a sentence the sentence goes as the save's reply with
  "[{label}, saved]". Test `a_key_pasted_into_the_reply_box_is_saved_and_never_sent_as_words`.
- [x] F4 A key that looks wrong: "… Nothing was sent or saved; paste it in the card on the thread
  to keep it anyway.", nothing sent or saved; the words stay in the box but are not its draft any
  more: not written to the drafts file, gone when the box folds or the thread is left, kept again
  once they change (`unkept`). Test
  `a_key_that_looks_wrong_in_the_reply_box_is_neither_sent_nor_saved_nor_kept`.
- [x] F5 A key for a sign-in of several variables goes as words, as the web window sends it. Test
  `a_code_typed_into_the_reply_box_goes_as_the_code` (its last case). See "Deviations" 10.
- [x] F6 Caught in `send_reply` before `hold_reply`: nothing drawn, held, kept as on its way, or
  written; with files, in `carried` on the words and their links once the files are saved. Tests
  (`held`, `answering.in_flight`, `answering.sending` empty),
  `a_key_with_files_in_the_box_is_caught_once_the_files_are_saved`.
- [x] F7 A refusal from the box's save is a line. Test.

## G. The writes

- [x] G1 `zero:cloud-key` `{ action: 'save', name, value, label, from?, reply? }`, one per
  variable in order, the first refusal stopping the rest (`send_all`), `reply` on the last only.
  Tests (exact payloads).
- [x] G2 "Live " or "Test-mode " before a Stripe key's label on a live card; a late key's as it
  is. Tests.
- [x] G3 The reply with the last key: the person's words; or, with files held and nothing left,
  `connectedReply` less "Carry on.", `heldWords`, the words, "Carry on." (which names the sign-in
  twice, as the web window's does: "Deviations" 9); a late key's `lateReply`. Tests.
- [x] G4 `zero:github-cloud-sign-in` `{ action: 'start' }`, then `{ action: 'token' }`; GitHub's
  page opened in the system's browser. Test.
- [x] G5 `zero:cloud-file-info` `{ path }`. Test.
- [x] G6 `zero:cloud-send-file` `{ product, id, path, hold, also? }`; late, `hold: false`, no
  `also`. Tests. A chosen file goes as its `path`, never `choose: true` ("Deviations" 3).
- [x] G7 `zero:cloud-send-code` `{ product, id, about, code, also? }`. Tests.
- [x] G8 "Do it from my computer." held three seconds and written on `zero:answer` as any reply.
  Test. See "Deviations" 6.
- [x] G9 The lines after each write, in the web window's words. Tests.
- [x] G10 `HELD`: per thread, for the window's life, let go once a save or a send carries it.
  Tests; picture `-held` (from the fixture's `cloudNeeds.held`).
- [x] G11 Keys just saved count at once. Test (the card says connected before the lists do).

## K. A key is never shown or kept

- [x] K1 Every key box is `.secure()` (its model too); the code's box is not. Test (no element's
  words or label holds the typed key); pictures `-warned`, `-aws`, `-saving` (dots).
- [x] K2 A key is in its box and in the write that saves it, nowhere else: never in a line, a
  log, the card's state past its box, the `Sent` the answer is filed under, or the drafts file
  once sent or refused; a token from gh goes from the engine's answer straight into its save.
  Read off the code (`Saving` holds names only); tests above. The reply box keeps its words as
  typed until Send, in both windows (the web window's `localStorage`, this one's drafts file).

## H. How it was measured

Each scene was photographed from the web renderer (`reference.mjs`, 1280 by 760, dark, 2x) and
drawn here from its fixture; each card's rectangle in the web's dump (`.need`) was compared with
this window's keyed card (`need-cloud~…` or `need-done~…`) in its `--tree`, and the two crops
looked at side by side. This window's column stands 15.6 points to the right of the web's (its
rail is the older, wider one: README, "What still differs"); every other number is the web's:

| Scene | Card(s), width by height at y, both windows |
|---|---|
| `thread-needs` | 640 by 270.8 at 287.95 |
| `thread-needs-several` | 640 by 229 at 252.35; 179 at 491.35; 194.8 at 680.35 |
| `thread-needs-two` | 640 by 229 at 253.59; 194.8 at 492.59 |
| `thread-needs-key-here` | 640 by 266.8 at 253.59 |
| `thread-needs-key` | 640 by 265.8 at 212 |
| `thread-needs-aws` | 640 by 353.8 at 212 |
| `thread-needs-warned` | 640 by 311.8 at 212 |
| `thread-needs-refused` | 640 by 311.8 at 287.95 |
| `thread-needs-saving` | 640 by 270.8 at 287.95 |
| `thread-needs-github` | 640 by 245.8 at 253.59 |
| `thread-needs-github-device` | 640 by 305.39 at 253.59 |
| `thread-needs-github-token` | 640 by 270.8 at 253.59 |
| `thread-needs-file`, `-file-missing` | 640 by 220.8 at 253.59 |
| `thread-needs-held` | 640 by 229 at 252.35; 77 at 491.35; 194.8 at 578.35 |
| `thread-needs-code` | 640 by 201.8 at 253.59 |
| `thread-needs-saved` | 640 by 77 at 287.95 |
| `thread-needs-answered`, `-sent-late` | 640 by 77 at 119.99, 206.99, 293.99, 380.99 |
| `thread-needs-late` | 640 by 204 at -7.01; 77 at 206.99, 293.99, 380.99 |
| `thread-needs-unfolded` | 640 by 229, 179, 194.8, each 0.49 higher than the web's (194.08, 433.08, 622.08) |
| `thread-needs-no-cloud` | no card; the code block the same in both |

In `thread-needs-unfolded` the three cards stand together 0.49 points higher: the page lands on its
foot, where this window's scroll rests half a point from the browser's (README, "What still
differs"). Inside the cards the crops agree to the eye but for the ground's colour (the web's
newer palette).

## Deviations for the owner to decide

1. **The AWS logo's letters are this window's own text.** Its file sets them as an SVG `<text>`,
   which the rasteriser behind `logos::of` draws none of, so `mark_of` sets them over the smile
   in the same system face. On a computer with neither Helvetica nor Arial they fall back to the
   window's own sans.
2. **E9.2**, above: no button takes the keys; a box does not select what it holds as it takes
   them.
3. **The file panel is this window's own.** "Choose another file" and "Choose the file" open the
   system's panel over this window, titled "Pick the file to send" as the app's is, and send the
   path picked (`path`, where the web window sends `choose: true` and the app opens its panel over
   the Electron window). Rune's panel takes no button label, so its button is the system's own
   ("Open"), not "Send this file". A panel that cannot open says "The file picker did not open:
   {why}" on the card, as the paper clip's does.
4. **A file's size is asked once a visit**, per thread and path, as the card comes up or is opened
   again; the web window asks again each time a card starts taking.
5. **The code's box** takes the spaces out and then keeps twelve; the web window's `maxLength`
   cuts what is typed at twelve before its spaces are taken out, so a pasted spaced code longer
   than twelve keeps more digits here.
6. **"Do it from my computer."** goes through this window's reply path, which writes the thread's
   own `permissionMode` with it (untouched chips); the web window's `answerWith` writes none. The
   thread runs as it did either way.
7. **A write that reaches no app** (no door) is said on the card in the web window's own
   refusal words; in the web window an IPC that throws leaves the card busy.
8. **A card keeps what it holds** when the page changes between the newest answer alone and the
   unfolded conversation (it is the same message); the web window's card is mounted anew.
9. **Ported as the web window does it, worth a look:** a key saved while a held file waits sends
   a reply that names the sign-in twice ("I connected Stripe. I connected Stripe. I sent …"),
   because `saveKeys` puts `connectedReply` before `heldWords`, which already holds it.
10. **Ported as the web window does it, worth a look:** a key pasted into the reply box for a
    sign-in of two variables (AWS) is sent as words, in plain sight, as the web window sends it.
