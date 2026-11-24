// A file the run asks for, its size answered by the engine (`zero:cloud-file-info`): where it
// is and how big, the fact, "Send from this computer" and "Choose another file".
import { needs } from './thread-needs.mjs';

export const FILE = 'The signed lease goes with the filing.\n\nAGENTBOX-NEEDS: file ~/Downloads/signed-lease.pdf | to attach it to the filing';
export const PATH = '~/Downloads/signed-lease.pdf';

// `more` is what else the scene does, as `needs` takes it; what the engine said of the file
// goes in the fixture with what `more.then` returns.
export const filed = (h, info, more = {}) => needs(h, {
  said: FILE,
  early: { cloudFileInfo: `async () => (${JSON.stringify(info)})` },
  ...more,
  then: async (h) => {
    const did = more.then ? (await more.then(h)) ?? {} : {};
    return { ...did, cloudNeeds: { ...(did.cloudNeeds ?? {}), fileInfo: { [PATH]: info } } };
  },
});

export const go = (h) => filed(h, { ok: true, size: 3 * 1024 * 1024 });
