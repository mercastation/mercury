// The code button pressed: the change beside the conversation (`CodeArtifact` in
// renderer/src/components/CodeArtifact.tsx, in the pane `DocPane.tsx` draws), the button lit.
// Up to a pane of 800 points the files are a picker over the column; wider, a tree beside it.
// The fixture says which change was open (`docOpen`), and whatever `then` did after.
import { open } from './thread.mjs';
import { changeFor, changePathFor, hexId, pressTheButton } from './change.mjs';

export async function openChange(h, { change = changeFor(), then = null, runs = null, unfold = false } = {}) {
  const out = await open(h, {
    unfold,
    runs,
    files: {},
    change,
    dress: hexId,
    then: async (h) => {
      await pressTheButton(h);
      return then ? (await then(h)) ?? {} : {};
    },
  });
  const id = Object.keys(out.threads)[0].split('/').pop();
  return { docOpen: changePathFor(id), ...out };
}

export const go = (h) => openChange(h);
