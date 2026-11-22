// Home with words typed into its box and not yet sent: the caret in it, Send lit.
export const WORDS = 'Add a dark mode toggle to the settings page';

// Puts the caret in Home's box, as a press on it does, and types.
export async function type(h, words = WORDS) {
  await h.evaluate(`document.querySelector('.hm-start textarea.tc-text').focus()`);
  await h.wait(200);
  if (words) await h.call('Input.insertText', { text: words });
  await h.wait(300);
}

export async function go(h) {
  await type(h);
  return { composing: WORDS };
}
