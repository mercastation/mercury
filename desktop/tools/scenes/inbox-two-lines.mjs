// The Inbox's first tab with one row of each state the two-line look draws its own way
// (`TWO LINES A THREAD` in renderer/src/threads/pages.css): two unread rows, whose dot stands
// level with the face; a row whose answer only says its title again, which keeps one line; a
// row whose faint words after the title are what its second line says, which go from the first
// (`.aside-moved`); a row with a comment on its link, whose words stay after the title; and a
// row kept at the top, with its pin. All invented.
import { go as toInbox } from './inbox.mjs';

const MIN = 60_000;

export function dress(snapshot, now) {
  const items = snapshot.items.map((i) => {
    const by = (source, ts) => ({ ts, source });
    switch (i.id) {
      // Its answer is its title, word for word: nothing to say under it.
      case 'w-readme7': return { ...i, result: i.title };
      // Handed back with words that are its answer's.
      case 'w-readme2': return { ...i, handedOver: { said: i.result } };
      // A comment on its link, after the title.
      case 'w-readme4': return { ...i, linkComments: [{ name: 'Ines', file: 'plan.md', text: 'Does this cover Android too?', ts: now - 5 * MIN }] };
      // Still open, waiting on the person, and kept at the top.
      case 'w-readme6': return { ...i, status: 'blocked', pinned: true, wrote: { ...i.wrote, status: by('agent', i.wrote.status.ts), pinned: by('founder', now - 9 * MIN) } };
      default: return i;
    }
  });
  return { ...snapshot, items };
}

// Two rows unread: everything since the start of time counts, and every other row was opened.
export async function go(h) {
  await h.evaluate(`(() => {
    const unread = ['w-readme0', 'w-readme3'];
    const seen = {};
    for (const i of window.__SNAP__?.items ?? []) if (!unread.includes(i.id)) seen[i.id] = Date.now();
    localStorage.setItem('zero.unreadSince', '1');
    localStorage.setItem('zero.seenAt', JSON.stringify(seen));
  })()`);
  await h.goto(h.origin);
  await h.evaluate('(async () => { await document.fonts.ready; })()');
  await h.wait(2000);
  await toInbox(h);
}
