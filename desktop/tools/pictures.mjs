// An invented project folder for the scenes that name files: flat pictures of different
// shapes, a folder of rounds, and a page, a note, a sheet, a film and a sound to link to.
//
//   node desktop/tools/pictures.mjs [folder]     # plants it and prints where
//
// A scene needs files the page can really load and the native window can really read. The
// page asks for a picture on the app's own scheme (`renderer/src/picture.ts`), which only the
// app answers, so `standIn` is a script for the page that answers those addresses from the
// planted files. The native window reads the same files off the disk, by the product's `dir`
// in the fixture. Nothing here is anybody's real file, and nothing outside the planted folder
// is touched. Each planting starts the folder again, so scenes that plant are photographed
// one after another, never two at once.
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import zlib from 'node:zlib';
import { fileURLToPath } from 'node:url';

/** Where the scenes plant their folder unless told otherwise. */
export const PLANTED = path.join(os.tmpdir(), 'mercury-desktop-scenes');

const chunk = (type, data) => {
  const body = Buffer.concat([Buffer.from(type, 'latin1'), data]);
  const out = Buffer.alloc(body.length + 8);
  out.writeUInt32BE(data.length, 0);
  body.copy(out, 4);
  out.writeUInt32BE(zlib.crc32(body) >>> 0, body.length + 4);
  return out;
};

/**
 * A PNG of one colour with what makes a wrong fit show in a comparison: a darker frame around
 * it, a light block in its top left corner and a dark disc in its middle. Cropped, stretched
 * or turned, one of the three lands somewhere else.
 */
export function picture(width, height, [r, g, b]) {
  const frame = Math.max(4, Math.round(Math.min(width, height) * 0.04));
  const block = Math.round(Math.min(width, height) * 0.22);
  const disc = Math.min(width, height) * 0.16;
  const dark = [r, g, b].map((v) => Math.round(v * 0.45));
  const light = [r, g, b].map((v) => Math.round(v + (255 - v) * 0.6));
  const row = Buffer.alloc(1 + width * 3);
  const rows = [];
  for (let y = 0; y < height; y += 1) {
    for (let x = 0; x < width; x += 1) {
      const edge = x < frame || y < frame || x >= width - frame || y >= height - frame;
      const corner = x >= frame * 2 && y >= frame * 2 && x < frame * 2 + block && y < frame * 2 + block;
      const middle = Math.hypot(x - width / 2, y - height / 2) < disc;
      const [pr, pg, pb] = edge || middle ? dark : corner ? light : [r, g, b];
      row[1 + x * 3] = pr; row[2 + x * 3] = pg; row[3 + x * 3] = pb;
    }
    rows.push(Buffer.from(row));
  }
  const head = Buffer.alloc(13);
  head.writeUInt32BE(width, 0);
  head.writeUInt32BE(height, 4);
  head[8] = 8; head[9] = 2;
  return Buffer.concat([
    Buffer.from([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]),
    chunk('IHDR', head), chunk('IDAT', zlib.deflateSync(Buffer.concat(rows))), chunk('IEND', Buffer.alloc(0)),
  ]);
}

// The pictures, by where they sit under the project's folder. `<thread>` is the id of the
// thread the scene opens, whose own designs folder is the first place a bare name is looked for.
const PICTURES = [
  // Wider than the column on any screen, so it is drawn from a smaller copy.
  ['designs/<thread>/checkout-wide.png', 2400, 1350, [52, 120, 196]],
  // Smaller than the column: drawn at its own size.
  ['designs/<thread>/receipt-small.png', 400, 300, [186, 92, 60]],
  // Taller than a window.
  ['designs/<thread>/phone-tall.png', 900, 1950, [70, 150, 104]],
  // Strips, short enough that several ways of naming a picture fit one window.
  ['designs/<thread>/total-line.png', 360, 64, [176, 108, 52]],
  ['designs/<thread>/tax-line.png', 300, 64, [92, 128, 190]],
  ['designs/<thread>/total-narrow.png', 240, 64, [120, 164, 92]],
  ['shots/cart-strip.png', 320, 64, [168, 84, 132]],
  // Three shapes for one row.
  ['shots/cart-a.png', 1200, 800, [150, 96, 178]],
  ['shots/cart-b.png', 800, 800, [196, 160, 62]],
  ['shots/cart-c.png', 600, 900, [64, 150, 160]],
  // A folder of rounds, the last drawn the newest.
  ...[1, 2, 3, 4, 5, 6, 7].map((n) => [`designs/rounds/round-${n}.png`, 1600, 1000, [40 + n * 26, 110 + n * 8, 200 - n * 20]]),
  // One level only: a round kept in a folder of its own is not one of the folder's pictures.
  ['designs/rounds/older/round-0.png', 800, 500, [120, 120, 120]],
];
// Ten screenshots the size a Retina Mac takes them, for measuring what a conversation full of
// pictures costs. Planted only when asked for (`large`): they take a few seconds to paint.
const LARGE = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10].map((n) => [`screens/screen-${String(n).padStart(2, '0')}.png`, 2880, 1800, [30 + n * 20, 200 - n * 12, 90 + n * 9]]);
// What a message links to and never draws. Their words do not matter, only that they are there.
const OTHERS = [
  ['designs/<thread>/checkout.html', '<!doctype html><title>Checkout</title><h1>Checkout</h1>'],
  // A page that is not a design: the web window frames a design under the message that names it.
  ['pages/checkout.html', '<!doctype html><title>Checkout</title><h1>Checkout</h1>'],
  ['notes/rounding.md', '# Rounding\n\nRound each tax line before the total is added up.\n'],
  ['exports/failed-orders.csv', 'order,total,failed\n1001,1049.99,yes\n1002,412.00,no\n'],
  ['reports/week-32.pdf', '%PDF-1.4\n% an invented report with nothing in it\n'],
  ['media/checkout-demo.mp4', 'an invented film with nothing in it'],
  ['media/voice-over.mp3', 'an invented sound with nothing in it'],
  ['designs/rounds/notes.txt', 'Seven rounds, the seventh the newest.\n'],
];

/**
 * Plants the invented project under `root` for the thread `thread` and says where everything
 * is: the project's folder (`dir`), its code folder (`repo`) and every file by its full path.
 */
export function plant({ root = PLANTED, thread = 'w-thread', large = false } = {}) {
  const dir = path.join(root, 'storefront');
  const repo = path.join(root, 'storefront-code');
  fs.rmSync(root, { recursive: true, force: true });
  const files = [];
  const write = (base, rel, bytes, at) => {
    const file = path.join(base, rel.replace('<thread>', thread));
    fs.mkdirSync(path.dirname(file), { recursive: true });
    fs.writeFileSync(file, bytes);
    // A fixed time each, a minute apart in the order planted, so "newest first" never changes.
    const when = new Date(Date.UTC(2026, 0, 5, 9, at));
    fs.utimesSync(file, when, when);
    files.push(file);
  };
  PICTURES.forEach(([rel, w, h, colour], n) => write(dir, rel, picture(w, h, colour), n));
  if (large) LARGE.forEach(([rel, w, h, colour], n) => write(dir, rel, picture(w, h, colour), 20 + n));
  OTHERS.forEach(([rel, words], n) => write(dir, rel, words, 40 + n));
  // And one picture a worker left in the code folder, where a short path is looked for last.
  write(repo, 'shots/from-the-code.png', picture(280, 64, [200, 80, 110]), 59);
  return { root, dir, repo, thread, files };
}

/** The pictures of one planted folder, newest first, as the app lists them (`main/folder-pictures.mts`). */
export function folderPictures(folder) {
  const pictures = fs.readdirSync(folder, { withFileTypes: true })
    .filter((e) => e.isFile() && /\.(png|jpe?g|gif|webp|avif)$/i.test(e.name))
    .map((e) => ({ file: path.join(folder, e.name), mtime: fs.statSync(path.join(folder, e.name)).mtimeMs }))
    .sort((a, b) => b.mtime - a.mtime || a.file.localeCompare(b.file));
  return { ok: true, folder, pictures: pictures.slice(0, 60).map((p) => p.file), total: pictures.length };
}

/**
 * A script for the page, to run before it loads: every address on the app's picture scheme
 * is answered from the planted files (200 with the bytes, or 404), and the bridge says which
 * pictures a planted folder holds. `arriving` leaves every picture unanswered for ever, for a
 * photograph of the place the page holds while one is on its way.
 */
export function standIn(planted, { arriving = false } = {}) {
  const pictures = {};
  for (const file of planted.files) if (/\.(png|jpe?g|gif|webp)$/i.test(file)) pictures[file] = fs.readFileSync(file).toString('base64');
  const folders = {};
  const walk = (folder) => {
    folders[folder] = folderPictures(folder);
    for (const e of fs.readdirSync(folder, { withFileTypes: true })) if (e.isDirectory()) walk(path.join(folder, e.name));
  };
  walk(planted.dir);
  return `(() => {
    const pictures = ${JSON.stringify(pictures)};
    const folders = ${JSON.stringify(folders)};
    const dir = ${JSON.stringify(planted.dir)};
    const real = window.fetch.bind(window);
    window.fetch = (input, init) => {
      const url = String(input?.url ?? input);
      const at = /^[a-z0-9-]+-img:\\/\\/file(\\/[^?#]*)/i.exec(url);
      if (!at) return real(input, init);
      if (${JSON.stringify(arriving)}) return new Promise(() => {});
      const bytes = pictures[decodeURIComponent(at[1])];
      if (!bytes) return Promise.resolve(new Response('Not there.', { status: 404 }));
      const raw = Uint8Array.from(atob(bytes), (c) => c.charCodeAt(0));
      return Promise.resolve(new Response(raw, { status: 200, headers: { 'content-type': 'image/png' } }));
    };
    window.__FOLDER_PICTURES__ = async ({ src } = {}) => {
      const named = String(src ?? '').replace(/\\/+$/, '');
      return folders[named.startsWith('/') ? named : dir + '/' + named] ?? { ok: false, error: named + ' could not be found.' };
    };
  })()`;
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const planted = plant({ root: process.argv[2] ?? PLANTED });
  console.log(`${planted.files.length} files under ${planted.root}`);
}
