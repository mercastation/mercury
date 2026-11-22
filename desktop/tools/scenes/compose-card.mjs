// N pressed on the Inbox: the New thread card over the page, with words typed into it.
import { go as toInbox } from './inbox.mjs';
import { press } from './thread-option-picked.mjs';
import { WORDS } from './home-typed.mjs';

// Opens the card from the Inbox and types into it. `words` empty leaves the placeholder.
export async function openCard(h, words = WORDS) {
  await toInbox(h);
  await press(h, 'n', 'KeyN', 78);
  await h.wait(500);
  if (words) await h.call('Input.insertText', { text: words });
  await h.wait(300);
}

// Presses one of the card's own controls: a field of its head by its label, or a chip of its
// bar by its title.
export async function pressOnCard(h, { field, chip }) {
  await h.evaluate(`(() => {
    const card = document.querySelector('.tc-card:not(.tc-inline)');
    const field = [...card.querySelectorAll('.tc-field')].find((f) => f.querySelector('.tc-label')?.textContent === ${JSON.stringify(field ?? '')});
    const chip = card.querySelector('.tc-bar [data-trigger][title=${JSON.stringify(chip ?? '')}]');
    (field?.querySelector('[data-trigger]') ?? chip).click();
  })()`);
  await h.wait(500);
}

export async function go(h) {
  await openCard(h);
  return { card: true, composing: WORDS };
}
