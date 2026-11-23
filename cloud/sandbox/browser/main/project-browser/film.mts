// WHAT AN AGENT DID IN ITS PROJECT'S BROWSER, KEPT AS A FILM.
//
// The browser is hidden, and it is signed in as the owner, so what an agent
// presses there is done in their name. The native window shows it: the page as
// the agent sees it, with a pointer where it pressed and a few words on what,
// live while it works and step by step afterwards. Mercury draws the pointer
// and the words over the picture (desktop/src/film.rs); nothing is drawn inside
// the page, so the site sees nothing.
//
// One folder per thread, under the project's pages (`<pages>/film/<thread>`):
// - `steps.jsonl`, one line when a step starts and one when it ends, by its
//   number: the tool, what it acted on, where on the page, which pictures.
// - `<n>a.jpg`, the page the moment before a step that presses something, and
//   `<n>b.jpg`, the page once a step that changes it is done.
// - `live.jpg`, the newest picture, replaced while a step runs.
//
// WHAT IT COSTS IS WHY IT IS BUILT THIS WAY. Nothing is filmed between steps,
// which is nearly all of an agent's time: it is thinking. During a step the
// page's own frames are taken from Chrome (a screencast) and at most four a
// second are written. Reading steps (a snapshot, a search of the page) take no
// picture at all: the page did not change. The picture after a step is taken
// once the agent has its answer, so filming never makes a call slower than the
// short look before a press. Each thread's folder and the project's are capped,
// oldest first (`FILM_CAPS`).
import fs from 'node:fs';
import path from 'node:path';
import type { CDPSession, Page } from 'playwright-core';

export const FILM_FOLDER = 'film';
export const STEPS_FILE = 'steps.jsonl';
export const LIVE_FILE = 'live.jpg';

export const FILM_CAPS = {
  /** A thread's pictures, past which its oldest steps lose theirs. */
  threadBytes: 48 * 1024 * 1024,
  /** All the project's films, past which the threads used longest ago are let go whole. */
  projectBytes: 384 * 1024 * 1024,
  /** And at most this many threads' films. */
  threads: 40,
};

/** The fewest milliseconds between two live pictures written: four a second. */
export const LIVE_EVERY_MS = 250;
/** How long after a step's answer its page is looked at again: a menu a press opened is drawn by then. */
export const AFTER_MS = 350;
/** The longest a look before a press may take, so a slow page never holds the agent up. */
const LOOK_MS = 800;
const QUALITY = 60;
const VIEW = { width: 1280, height: 720 };

const safe = (name: string): string => name.replace(/[^a-zA-Z0-9._-]/g, '_').replace(/^\.+/, '_') || '_';

/** The project's films, under its pages folder. */
export const filmRoot = (pages: string): string => path.join(pages, FILM_FOLDER);
export const filmDir = (pages: string, thread: string): string => path.join(filmRoot(pages), safe(thread));

/** What a step is to the film: filmed while it runs, looked at after, or only named. */
export type Kind = 'acts' | 'reads';

/** The tools the film names, and which of them change the page. A tool missing here is not in the film. */
const KINDS: Record<string, Kind> = {
  browser_navigate: 'acts',
  browser_navigate_back: 'acts',
  browser_click: 'acts',
  browser_hover: 'acts',
  browser_type: 'acts',
  browser_press_key: 'acts',
  browser_select_option: 'acts',
  browser_fill_form: 'acts',
  browser_handle_dialog: 'acts',
  browser_wait_for: 'acts',
  browser_tabs: 'acts',
  browser_snapshot: 'reads',
  browser_find: 'reads',
  browser_take_screenshot: 'reads',
};

export function kindOf(tool: string, args: Record<string, unknown>): Kind | null {
  const kind = KINDS[tool] ?? null;
  // Listing or closing tabs shows nothing of a page.
  if (tool === 'browser_tabs' && args.action !== 'new' && args.action !== 'select') return null;
  return kind;
}

const clip = (text: string, most = 60): string => {
  const one = text.replace(/\s+/g, ' ').trim();
  return one.length > most ? `${one.slice(0, most - 1).trimEnd()}…` : one;
};
const str = (v: unknown): string => (typeof v === 'string' ? v : '');

/** An address as a person reads it: its host and path, without the scheme or a trailing slash. */
export function shortAddress(address: unknown): string {
  try {
    const url = new URL(String(address));
    const rest = `${url.pathname === '/' ? '' : url.pathname}${url.search}`;
    return clip(`${url.hostname.replace(/^www\./, '')}${rest}`);
  } catch { return clip(str(address)); }
}

/** The element a step names, as the agent described it. */
function described(args: Record<string, unknown>): string {
  return clip(str(args.element));
}

/**
 * What a step acted on, in a few words: the element the agent named, the
 * address, the key. Never what was typed, which may be anything.
 */
export function whatOf(tool: string, args: Record<string, unknown>): string {
  switch (tool) {
    case 'browser_navigate': return shortAddress(args.url);
    case 'browser_click':
    case 'browser_hover':
    case 'browser_type':
    case 'browser_select_option':
    case 'browser_take_screenshot':
      return described(args);
    case 'browser_fill_form': {
      const fields = Array.isArray(args.fields) ? args.fields : [];
      return clip(fields.map((f) => str((f as Record<string, unknown>)?.name) || str((f as Record<string, unknown>)?.element)).filter(Boolean).join(', '));
    }
    case 'browser_press_key': return clip(str(args.key), 24);
    case 'browser_tabs': return args.action === 'new' ? shortAddress(args.url) : Number.isFinite(Number(args.index)) ? `tab ${Number(args.index) + 1}` : '';
    case 'browser_find': return clip(str(args.text) || str(args.regex));
    case 'browser_wait_for': return clip(str(args.text) || str(args.textGone) || (Number.isFinite(Number(args.time)) ? `${Number(args.time)} s` : ''));
    case 'browser_handle_dialog': return args.accept === false ? 'dismissed' : 'accepted';
    default: return '';
  }
}

/** The element a step presses, as Playwright finds it: a reference from the page's snapshot, or a selector. */
export function targetOf(tool: string, args: Record<string, unknown>): string | null {
  if (tool === 'browser_fill_form') {
    const first = Array.isArray(args.fields) ? args.fields[0] as Record<string, unknown> | undefined : undefined;
    return str(first?.target) || null;
  }
  if (tool === 'browser_take_screenshot' || tool === 'browser_click' || tool === 'browser_hover' || tool === 'browser_type' || tool === 'browser_select_option') return str(args.target) || null;
  return null;
}

/** A snapshot's reference (`e12`, `f1e3`) is found by Playwright's own `aria-ref`; anything else is a selector. */
export const locatorFor = (target: string): string => (/^(f\d+)?e\d+$/.test(target) ? `aria-ref=${target}` : target);

/** One line of `steps.jsonl`. A step's first line has `tool`; its second has `end`. */
export interface StepStart {
  n: number;
  tool: string;
  what: string;
  at: number;
  url?: string;
  /** Where it pressed, in the page's pixels, and the page's size. */
  box?: [number, number, number, number];
  view?: [number, number];
  before?: string;
}
export interface StepEnd {
  n: number;
  end: number;
  after?: string;
  url?: string;
  failed?: boolean;
}

/** What the host hands the film for one step. */
export interface Take {
  /** The step's answer came back: the page is looked at once more, off the agent's time. */
  end(page: Page | null, failed: boolean): void;
}

interface Thread {
  dir: string;
  n: number;
  bytes: number;
  /** Each step's pictures and their sizes, oldest first, for the cap. */
  pictures: { n: number; files: string[]; bytes: number }[];
  /** The last thing written, so lines land in order. */
  writing: Promise<void>;
}

const sizeOf = (file: string): number => { try { return fs.statSync(file).size; } catch { return 0; } };

/** Every thread's film under `root`, newest first, with its size. */
function films(root: string): { dir: string; at: number; bytes: number }[] {
  let names: string[];
  try { names = fs.readdirSync(root); } catch { return []; }
  return names.map((name) => {
    const dir = path.join(root, name);
    let bytes = 0;
    let at = 0;
    try {
      for (const file of fs.readdirSync(dir)) bytes += sizeOf(path.join(dir, file));
      at = fs.statSync(path.join(dir, STEPS_FILE)).mtimeMs;
    } catch { /* an empty or broken folder sorts last */ }
    return { dir, at, bytes };
  }).sort((a, b) => b.at - a.at);
}

/** Lets go of the project's oldest films past its caps. The thread being filmed is kept whatever it holds. */
export function trimFilms(root: string, { keep = '', caps = FILM_CAPS }: { keep?: string; caps?: typeof FILM_CAPS } = {}): string[] {
  const gone: string[] = [];
  let total = 0;
  films(root).forEach((film, i) => {
    total += film.bytes;
    if (film.dir === keep || (i < caps.threads && total <= caps.projectBytes)) return;
    fs.rmSync(film.dir, { recursive: true, force: true });
    gone.push(film.dir);
  });
  return gone;
}

/** Reads a thread's film back into the numbers and sizes the next steps go on from. */
function reopen(dir: string): Pick<Thread, 'n' | 'bytes' | 'pictures'> {
  const pictures = new Map<number, { n: number; files: string[]; bytes: number }>();
  let n = 0;
  let text = '';
  try { text = fs.readFileSync(path.join(dir, STEPS_FILE), 'utf8'); } catch { /* a new film */ }
  for (const line of text.split('\n')) {
    let step: Partial<StepStart & StepEnd>;
    try { step = JSON.parse(line); } catch { continue; }
    if (!Number.isInteger(step.n)) continue;
    n = Math.max(n, step.n!);
    for (const file of [step.before, step.after]) {
      if (!file) continue;
      const bytes = sizeOf(path.join(dir, file));
      if (!bytes) continue;
      const held = pictures.get(step.n!) ?? { n: step.n!, files: [], bytes: 0 };
      held.files.push(file);
      held.bytes += bytes;
      pictures.set(step.n!, held);
    }
  }
  const list = [...pictures.values()].sort((a, b) => a.n - b.n);
  return { n, pictures: list, bytes: list.reduce((sum, p) => sum + p.bytes, 0) };
}

export interface FilmDeps {
  /** The project's pages folder. */
  pages: () => string | null;
  now?: () => number;
  caps?: typeof FILM_CAPS;
  /** A CDP session on a page, for its pictures. */
  cdp?: (page: Page) => Promise<CDPSession>;
  log?: (line: string) => void;
}

/**
 * The film of one project's browser. `begin` is called before each call a
 * thread makes and the step it hands back is ended after it; a tool the film
 * does not name gets nothing.
 */
export function projectFilm({ pages, now = Date.now, caps = FILM_CAPS, cdp = (page) => page.context().newCDPSession(page), log = () => {} }: FilmDeps) {
  const threads = new Map<string, Thread>();
  const sessions = new WeakMap<Page, Promise<CDPSession>>();
  let trimmed = false;

  const sessionOf = (page: Page): Promise<CDPSession> => {
    let had = sessions.get(page);
    if (!had) {
      had = cdp(page);
      sessions.set(page, had);
      page.once('close', () => sessions.delete(page));
    }
    return had;
  };

  function threadFilm(thread: string): Thread | null {
    const root = pages();
    if (!root) return null;
    const dir = filmDir(root, thread);
    const had = threads.get(thread);
    if (had?.dir === dir) return had;
    fs.mkdirSync(dir, { recursive: true });
    if (!trimmed) {
      trimmed = true;
      try { trimFilms(filmRoot(root), { keep: dir, caps }); } catch (err) { log(`could not trim the films: ${(err as Error).message}`); }
    }
    const film: Thread = { dir, ...reopen(dir), writing: Promise.resolve() };
    threads.set(thread, film);
    return film;
  }

  /** Writes in the order asked, after whatever this thread's film was writing. */
  const inOrder = (film: Thread, work: () => Promise<void>): Promise<void> => {
    film.writing = film.writing.then(work).catch((err: unknown) => log(`the film could not write: ${(err as Error)?.message ?? err}`));
    return film.writing;
  };

  const line = (film: Thread, step: StepStart | StepEnd): Promise<void> =>
    fs.promises.appendFile(path.join(film.dir, STEPS_FILE), `${JSON.stringify(step)}\n`);

  /** Replaces the live picture whole: a reader never meets half a file. */
  async function live(film: Thread, jpeg: Buffer): Promise<void> {
    const file = path.join(film.dir, LIVE_FILE);
    const part = `${file}.${process.pid}`;
    await fs.promises.writeFile(part, jpeg);
    await fs.promises.rename(part, file);
  }

  /** Keeps a step's picture, and lets the oldest steps' go past the thread's cap. */
  async function keep(film: Thread, n: number, name: string, jpeg: Buffer): Promise<void> {
    await fs.promises.writeFile(path.join(film.dir, name), jpeg);
    const held = film.pictures.find((p) => p.n === n);
    if (held) { held.files.push(name); held.bytes += jpeg.length; } else film.pictures.push({ n, files: [name], bytes: jpeg.length });
    film.bytes += jpeg.length;
    while (film.bytes > caps.threadBytes && film.pictures.length > 1) {
      const oldest = film.pictures.shift()!;
      film.bytes -= oldest.bytes;
      await Promise.all(oldest.files.map((f) => fs.promises.rm(path.join(film.dir, f), { force: true })));
    }
  }

  async function picture(page: Page): Promise<Buffer | null> {
    try {
      const session = await sessionOf(page);
      const shot = await session.send('Page.captureScreenshot', { format: 'jpeg', quality: QUALITY, optimizeForSpeed: true } as never) as { data: string };
      return Buffer.from(shot.data, 'base64');
    } catch { return null; }
  }

  /** Where the element a step presses is on the page, brought into view first as the press itself would. */
  async function boxOf(page: Page, target: string): Promise<[number, number, number, number] | null> {
    try {
      const element = page.locator(locatorFor(target)).first();
      await element.scrollIntoViewIfNeeded({ timeout: LOOK_MS });
      const box = await element.boundingBox({ timeout: LOOK_MS });
      return box ? [Math.round(box.x), Math.round(box.y), Math.round(box.width), Math.round(box.height)] : null;
    } catch { return null; }
  }

  /** The page's own frames while a step runs, the newest written at most four times a second. */
  async function screencast(film: Thread, page: Page): Promise<() => Promise<void>> {
    let session: CDPSession;
    try { session = await sessionOf(page); } catch { return async () => {}; }
    let last = 0;
    let writing: Promise<void> | null = null;
    const onFrame = (frame: { data: string; sessionId: number }) => {
      session.send('Page.screencastFrameAck', { sessionId: frame.sessionId }).catch(() => {});
      const at = now();
      if (writing || at - last < LIVE_EVERY_MS) return;
      last = at;
      writing = live(film, Buffer.from(frame.data, 'base64')).catch(() => {}).finally(() => { writing = null; });
    };
    session.on('Page.screencastFrame', onFrame);
    try {
      await session.send('Page.startScreencast', { format: 'jpeg', quality: QUALITY, maxWidth: VIEW.width, maxHeight: VIEW.height, everyNthFrame: 1 });
    } catch { session.off('Page.screencastFrame', onFrame); return async () => {}; }
    return async () => {
      session.off('Page.screencastFrame', onFrame);
      await session.send('Page.stopScreencast').catch(() => {});
      await writing;
    };
  }

  /**
   * A step starting. For a press, the element is found and the page is looked
   * at first, which is the one wait filming adds to an agent's call; then the
   * page's frames are taken until the step ends.
   */
  async function begin(thread: string, page: Page | null, tool: string, args: Record<string, unknown>): Promise<Take | null> {
    const kind = kindOf(tool, args);
    if (!kind) return null;
    const film = threadFilm(thread);
    if (!film) return null;
    await film.writing;
    const n = film.n += 1;
    const step: StepStart = { n, tool, what: whatOf(tool, args), at: now() };
    const target = targetOf(tool, args);
    if (page && !page.isClosed()) {
      step.url = page.url();
      const viewport = page.viewportSize() ?? VIEW;
      step.view = [viewport.width, viewport.height];
      if (target && kind === 'acts') {
        const box = await boxOf(page, target);
        const before = box ? await picture(page) : null;
        if (box) step.box = box;
        if (before) {
          step.before = `${n}a.jpg`;
          await inOrder(film, async () => { await keep(film, n, step.before!, before); await live(film, before); });
        }
      } else if (target && onlyWhere(tool)) {
        const box = await boxOf(page, target);
        if (box) step.box = box;
      }
    }
    void inOrder(film, () => line(film, step));
    const stop = kind === 'acts' && page && !page.isClosed() ? await screencast(film, page) : null;
    return {
      end(after: Page | null, failed: boolean) {
        void inOrder(film, async () => {
          if (stop) {
            await new Promise((resolve) => setTimeout(resolve, AFTER_MS));
            await stop();
          }
          const done: StepEnd = { n, end: now(), ...(failed ? { failed: true } : {}) };
          const shown = after && !after.isClosed() ? after : null;
          if (shown) done.url = shown.url();
          if (kind === 'acts' && shown) {
            const jpeg = await picture(shown);
            if (jpeg) {
              done.after = `${n}b.jpg`;
              await keep(film, n, done.after, jpeg);
              await live(film, jpeg);
            }
          }
          await line(film, done);
        });
      },
    };
  }

  return {
    begin,
    /** Waits for what the film is still writing, for a host closing and for tests. */
    settled: async () => { await Promise.all([...threads.values()].map((t) => t.writing)); },
  };
}

/** A picture of an element only names where it is: nothing is pressed, so no picture is taken before it. */
const onlyWhere = (tool: string): boolean => tool === 'browser_take_screenshot';

export type ProjectFilm = ReturnType<typeof projectFilm>;
