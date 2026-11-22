// The Inbox with its View and filters menu open (`DisplayMenu` in
// renderer/src/threads/Pages.tsx), on the display the page opens on: a list, by priority, no
// filter. The button at the head strip's right end is pressed, as a hand presses it.
//
// The page reads its display from its own storage when it starts (`readDisplay('inbox')` in
// App.tsx), so a scene that wants another one writes it there and loads the page again
// (`withDisplay`). The fixture carries the display (`display`) and which menu was open
// (`inboxMenu`), so the native window is drawn on the same.
import { go as toInbox } from './inbox.mjs';

export const DEFAULT = { view: 'list', sort: 'priority', priorities: [], projects: [], updated: 'any' };

/** Loads the page again on `display`, as one remembered from an earlier launch, and goes to the Inbox. */
export async function withDisplay(h, display) {
  await h.evaluate(`localStorage.setItem('threads.display.inbox', ${JSON.stringify(JSON.stringify({ ...display, v: 2 }))})`);
  await h.goto(h.origin);
  await h.evaluate('(async () => { await document.fonts.ready; })()');
  await h.wait(2000);
  await toInbox(h);
  return { display };
}

/** Presses the View and filters button (the one of the strip's square buttons that is not the cloud's). */
export async function openDisplay(h) {
  await h.evaluate(`document.querySelector('.th-right .th-disp:not(.th-cloud)')?.click()`);
  await h.wait(400);
  return { inboxMenu: 'display' };
}

/** Presses a button of the open menu by its words, on the line named `line`. */
export async function press(h, line, words) {
  await h.evaluate(`(() => {
    const lines = [...document.querySelectorAll('.th-pop .line')];
    const on = lines.find((l) => l.querySelector('.lab')?.textContent.trim().toLowerCase() === ${JSON.stringify(line.toLowerCase())});
    const button = [...(on?.querySelectorAll('button') ?? [])].find((b) => b.textContent.trim() === ${JSON.stringify(words)});
    button?.click();
  })()`);
  await h.wait(300);
}

export async function go(h) {
  await toInbox(h);
  return { display: DEFAULT, ...(await openDisplay(h)) };
}
