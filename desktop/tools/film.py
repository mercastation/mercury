"""Hold a film of the native window's working line against the web window's, frame by frame.

    python3 desktop/tools/film.py <web frames dir> <native frames dir> <web dump.json> [fps]

The web frames are `reference.mjs`'s with MERCURY_FILM set, of the line's region; the native
ones are `mercury --out`'s, whole windows. Both at 2x. The lattice's nine cells and the word are
found in the web window's dump of the same scene, so each film is read at the same points.

For each cell, the film is read as an opacity, frame by frame: how far its middle is from the
ground towards the ink. The two films start at different moments of the loop, so the native
one is slid along the web one to where they agree best, and what is left is how differently
they move: the mean and largest difference, in opacity, over every frame of every cell.

For the word, each frame is read against the frame where the band is furthest off it, column by
column: how much of the word's ink shows. That gives the band's middle (where least shows), how
deep it is, and how fast it moves, which are compared the same way.
"""
import json
import os
import sys

import numpy as np
from PIL import Image


def frames(folder, prefix):
    names = sorted(n for n in os.listdir(folder) if n.startswith(prefix) and n.endswith('.png'))
    return [os.path.join(folder, n) for n in names]


def gray(path, box=None):
    im = Image.open(path).convert('RGB')
    if box:
        im = im.crop(box)
    return np.asarray(im, dtype=np.float64).mean(axis=2)


def main():
    web_dir, native_dir, dump_path = sys.argv[1:4]
    fps = float(sys.argv[4]) if len(sys.argv) > 4 else 60.0
    dump = json.load(open(dump_path))
    nodes = dump['nodes']
    cells = [n for n in nodes if 'live-lattice-cell' in str(n['cls'])]
    word = next(n for n in nodes if 'live-word' in str(n['cls']))
    crop = [float(v) for v in os.environ.get('FILM_CROP', '220,602,200,34').split(',')]
    web = [gray(p) for p in frames(web_dir, '')]
    # The native window's frames are whole; cut the same region out of each, at 2x. Its first
    # two frames are before the line was reported on screen, and stand still.
    skip = int(os.environ.get('FILM_SKIP', '2'))
    box = tuple(int(round(v * 2)) for v in (crop[0], crop[1], crop[0] + crop[2], crop[1] + crop[3]))
    native = [gray(p, box) for p in frames(native_dir, 'frame_')[skip:]]

    def at(img, x, y, r=1.5):
        cx, cy = (x - crop[0]) * 2, (y - crop[1]) * 2
        return img[int(cy - r):int(cy + r) + 1, int(cx - r):int(cx + r) + 1].mean()

    # The ground and the ink, off the web's first frame: a point beside the mark, and the
    # brightest cell middle seen anywhere in either film.
    ground = at(web[0], crop[0] + 2, crop[1] + 2)
    middles = [(c['rect'][0] + c['rect'][2] / 2, c['rect'][1] + c['rect'][3] / 2) for c in cells]
    ink = max(max(at(f, x, y) for f in web + native) for x, y in middles)

    def opacity(img, x, y):
        return (at(img, x, y) - ground) / max(ink - ground, 1e-6)

    web_cells = np.array([[opacity(f, x, y) for x, y in middles] for f in web])
    native_cells = np.array([[opacity(f, x, y) for x, y in middles] for f in native])

    # Slide the native film along the web one to where the cells agree best.
    n = min(len(web_cells), len(native_cells) - 100)
    best = min(range(0, 100), key=lambda s: np.abs(web_cells[:n] - native_cells[s:s + n]).mean())
    diff = np.abs(web_cells[:n] - native_cells[best:best + n])
    print(f'lattice: {n} frames at {fps:g} a second, the native film {best} frames along')
    print(f'  opacity, mean difference {diff.mean():.3f}, largest {diff.max():.3f} (of 1)')
    for k, (x, y) in enumerate(middles):
        lo_w, hi_w = web_cells[:n, k].min(), web_cells[:n, k].max()
        lo_n, hi_n = native_cells[best:best + n, k].min(), native_cells[best:best + n, k].max()
        print(f'  cell {k}: web {lo_w:.2f} to {hi_w:.2f}, native {lo_n:.2f} to {hi_n:.2f}, mean difference {diff[:, k].mean():.3f}')

    # The period: where a cell's film matches itself again best.
    def period(series):
        lags = range(60, min(120, len(series) - 20))
        return min(lags, key=lambda lag: np.abs(series[lag:] - series[:-lag]).mean())

    print(f'  one turn: web {period(web_cells[:, 0]) / fps:.3f} s, native {period(native_cells[:, 0]) / fps:.3f} s')

    # The word: each column's ink against the frame the band is furthest off it.
    wx0, wy0, ww, wh = word['rect']
    cols = lambda img: img[int((wy0 - crop[1]) * 2):int((wy0 + wh - crop[1]) * 2), int((wx0 - crop[0]) * 2):int((wx0 + ww - crop[0]) * 2)]

    def shown(film):
        words = [cols(f) - ground for f in film]
        whole = max(words, key=lambda w: w.sum())
        weight = np.clip(whole, 0, None)
        return [(w * weight).sum(axis=0) / np.maximum((whole * weight).sum(axis=0), 1e-6) for w in words], weight.sum(axis=0) > weight.sum(axis=0).max() * 0.05

    web_shown, web_inked = shown(web)
    native_shown, native_inked = shown(native)

    def band(profile, inked):
        p = np.where(inked, profile, np.nan)
        if np.nanmin(p) > 0.97:
            return None
        return int(np.nanargmin(p)) / 2, float(np.nanmin(p))

    web_band = [band(p, web_inked) for p in web_shown]
    native_band = [band(p, native_inked) for p in native_shown]

    def speed(bands):
        pts = [(k / fps, b[0]) for k, b in enumerate(bands) if b and 5 < b[0] < ww - 5]
        if len(pts) < 4:
            return float('nan')
        t, x = np.array(pts).T
        # One crossing: keep the longest run of frames with the band going right.
        return np.polyfit(t, x, 1)[0]

    deepest = lambda bands: min((b[1] for b in bands if b), default=1.0)
    print(f'word: {ww:.1f} points wide')
    print(f'  the band on the word for {sum(1 for b in web_band if b)} of {len(web_band)} web frames, {sum(1 for b in native_band if b)} of {len(native_band)} native frames')
    print(f'  its middle moves {speed(web_band):.1f} points a second on the web, {speed(native_band):.1f} here')
    print(f'  the faintest the word gets: web {deepest(web_band):.3f}, native {deepest(native_band):.3f} of its ink')
    # The band's shape: the word's profile with the band's middle on the word's middle.
    def shape(film_shown, bands):
        mid = min((k for k, b in enumerate(bands) if b), key=lambda k: abs(bands[k][0] - ww / 2), default=None)
        return None if mid is None else film_shown[mid]
    sw, sn = shape(web_shown, web_band), shape(native_shown, native_band)
    if sw is not None and sn is not None:
        both = web_inked & native_inked
        print(f'  with its middle on the word\'s, the profile differs by {np.abs(sw - sn)[both].mean():.3f} on average, {np.abs(sw - sn)[both].max():.3f} at most')


if __name__ == '__main__':
    main()
