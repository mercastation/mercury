// From Home to the Inbox page's In progress tab.
import { go as toInbox } from './inbox.mjs';

// Press one of the page's tabs by its word, then wait until every row of it is drawn: a long
// list draws its first forty rows at once and the rest a few frames later.
export async function tab(h, label) {
  await toInbox(h);
  await h.evaluate(`(() => {
    const tabs = [...document.querySelectorAll('.tm-tabs button.tm-tab')];
    tabs.find((b) => b.firstChild?.textContent === ${JSON.stringify(label)})?.click();
  })()`);
  const rows = () => h.evaluate(`document.querySelectorAll('.list.th-table .row:not(.th-head)').length`);
  let seen = -1;
  for (let n = 0; n < 40; n += 1) {
    await h.wait(250);
    const now = await rows();
    if (now === seen) break;
    seen = now;
  }
}

export async function go(h) {
  await tab(h, 'In progress');
}
