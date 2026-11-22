// A picture in a conversation pressed: the file opened big over the window (`openBig` in
// renderer/src/components/Focus.tsx, which draws `ThreadArtifacts` and so `ArtifactView`). The
// thread is `thread-files`'s, whose agent drew the total line twice, so the big view opens on
// the newest of its two versions with the version line over it.
//
// The page finds a file through the app (`zero:open-artifact` asked to resolve) and draws a
// picture opened big from an `<img>` on the app's own scheme, neither of which the harness
// answers, so once the thread is open both are answered in the page from the planted folder:
// the finder by the project's folder, the picture through the planted files the page's fetch
// already serves (`standIn` in ../pictures.mjs).
import { open } from './thread.mjs';
import { FOUND, SHIPPED } from './thread-files.mjs';

/** The page's finder and its big pictures, answered from the planted folder. */
export async function findsFiles(h) {
  await h.evaluate(`(() => {
    const dirOf = (product) => (window.__SNAP__.products.find((p) => p.slug === product) ?? {}).dir ?? '';
    window.zero.openArtifact = async ({ product, src, mode } = {}) => {
      const p = String(src ?? '').replace(/^file:\\/\\//, '').split(/[?#]/)[0].replace(/^\\.\\//, '');
      return mode === 'resolve' ? { ok: true, opened: p.startsWith('/') ? p : dirOf(product) + '/' + p } : { ok: true };
    };
    const scheme = /^[a-z0-9-]+-img:\\/\\/file\\//i;
    const set = Element.prototype.setAttribute;
    const draw = (img, url) => fetch(url.split('?')[0]).then((r) => (r.ok ? r.blob() : null)).then((b) => { if (b) set.call(img, 'src', URL.createObjectURL(b)); });
    Element.prototype.setAttribute = function (name, value) {
      if (this instanceof HTMLImageElement && name === 'src' && scheme.test(String(value))) { draw(this, String(value)); return; }
      return set.call(this, name, value);
    };
    const src = Object.getOwnPropertyDescriptor(HTMLImageElement.prototype, 'src');
    Object.defineProperty(HTMLImageElement.prototype, 'src', { ...src, set(value) { if (scheme.test(String(value))) draw(this, String(value)); else src.set.call(this, value); } });
  })()`);
}

/**
 * Presses the `at`-th picture of the conversation and says so for the native window
 * (`pictureOpen`): the path the press opened and the pictures of its part of the thread, in
 * the order the page draws them (`picturesFor` in renderer/src/components/PictureView.tsx).
 */
export async function pressPicture(h, at = 0) {
  const pressed = await h.evaluate(`(() => {
    const door = document.querySelectorAll('.inline-image-open')[${at}];
    const own = door?.closest('[data-picture]');
    const part = own?.closest('[data-exchange]');
    const doors = [...(part ?? document).querySelectorAll('[data-picture]')].filter((d) => d.querySelector('img'));
    return own ? { path: own.dataset.picture, turn: doors.map((d) => d.dataset.picture) } : null;
  })()`);
  await h.evaluate(`document.querySelectorAll('.inline-image-open')[${at}]?.click()`);
  await h.wait(1500);
  return { pictureOpen: pressed };
}

export const go = (h) => open(h, {
  unfold: false,
  found: FOUND,
  shipped: SHIPPED,
  files: {},
  then: async (h) => {
    await findsFiles(h);
    return pressPicture(h, 0);
  },
});
