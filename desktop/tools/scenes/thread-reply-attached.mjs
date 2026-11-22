// A whole conversation whose person answered with a picture and a file attached: the lines the
// app wrote for them under the words (`attachmentLinks`) are drawn as the picture and the link
// they are, and the words stay as typed (`renderer/src/typed-message.ts`). The two files are in
// the invented folder's `attachments/`, where the app saves what is attached (`../pictures.mjs`).
import { open } from './thread.mjs';

export const REPLY = `Go with 1. The receipt should read like this one.

![pasted-41207.png](attachments/mwrd0001-ab12cd-pasted-41207.png)
[week-32.pdf](attachments/mwrd0002-ef34gh-week-32.pdf)`;

export const go = (h) => open(h, { unfold: true, files: {}, reply: REPLY });
