"""Compare the real Mercury screen with the native one, pixel by pixel.

  python compare.py <real.png> <native.png> <out dir> [name]

Prints how far apart they are overall and per region, and writes a side-by-side
picture and a difference map (differences shown 4x stronger).
"""
import sys, os
import numpy as np
from PIL import Image, ImageDraw

real_p, new_p, out = sys.argv[1], sys.argv[2], sys.argv[3]
name = sys.argv[4] if len(sys.argv) > 4 else 'home'
os.makedirs(out, exist_ok=True)
A = Image.open(real_p).convert('RGB')
B = Image.open(new_p).convert('RGB')
print('sizes', A.size, B.size)
if A.size != B.size:
    B = B.resize(A.size, Image.LANCZOS)
a = np.asarray(A).astype(np.int16)
b = np.asarray(B).astype(np.int16)
d = np.abs(a - b)
mx = d.max(axis=2)

def ssim(x, y, k=8):
    # Block SSIM on luminance, 8x8 blocks.
    lum = lambda im: (0.2126 * im[..., 0] + 0.7152 * im[..., 1] + 0.0722 * im[..., 2]).astype(np.float64)
    X, Y = lum(x), lum(y)
    h, w = (X.shape[0] // k) * k, (X.shape[1] // k) * k
    X = X[:h, :w].reshape(h // k, k, w // k, k).swapaxes(1, 2).reshape(-1, k * k)
    Y = Y[:h, :w].reshape(h // k, k, w // k, k).swapaxes(1, 2).reshape(-1, k * k)
    mx_, my_ = X.mean(1), Y.mean(1)
    vx, vy = X.var(1), Y.var(1)
    cov = ((X - mx_[:, None]) * (Y - my_[:, None])).mean(1)
    c1, c2 = (0.01 * 255) ** 2, (0.03 * 255) ** 2
    s = ((2 * mx_ * my_ + c1) * (2 * cov + c2)) / ((mx_ ** 2 + my_ ** 2 + c1) * (vx + vy + c2))
    return float(s.mean())

def stats(label, box=None):
    if box:
        x, y, w, h = [int(round(v * 2)) for v in box]
        dd, m, aa, bb = d[y:y + h, x:x + w], mx[y:y + h, x:x + w], a[y:y + h, x:x + w], b[y:y + h, x:x + w]
    else:
        dd, m, aa, bb = d, mx, a, b
    n = m.size
    if not n:
        # A region drawn for a taller or wider window than this picture is of.
        print(f"{label:26} outside this picture")
        return
    print(f"{label:26} mean diff {dd.mean():6.2f}/255  | pixels off by >8: {100 * (m > 8).sum() / n:5.1f}%  >24: {100 * (m > 24).sum() / n:5.1f}%  >64: {100 * (m > 64).sum() / n:5.1f}%  | similarity {ssim(aa, bb):.3f}")

stats('whole window')
# Regions, in points (the pictures are 2x). A scene names its own in
# tools/scenes/<name>.regions.json, as a list of [label, [x, y, w, h]]; Home's are here.
HOME = [
    ('rail', (0, 40, 82, 720)),
    ('top strip + arrows', (82, 40, 1176, 64)),
    ('title', (290, 165, 420, 50)),
    ('doors', (286, 220, 250, 40)),
    ('composer', (286, 274, 768, 160)),
    ('needs you card', (286, 450, 432, 288)),
    ('in progress card', (732, 450, 322, 288)),
    ('sky, left of column', (90, 300, 190, 430)),
    ('sky, right of column', (1060, 300, 190, 430)),
]
import json
# A scene with no regions of its own is measured by those of the scene it is a kind of:
# `inbox-later` and `inbox-cloud-count` by `inbox`'s.
scenes = os.path.join(os.path.dirname(os.path.abspath(__file__)), 'scenes')
parts = name.split('-')
own = next((p for p in (os.path.join(scenes, '-'.join(parts[:n]) + '.regions.json') for n in range(len(parts), 0, -1)) if os.path.exists(p)), None)
regions = [(label, tuple(box)) for label, box in json.load(open(own))] if own else (HOME if name.startswith('home') else [])
for label, box in regions:
    stats(label, box)

# Side by side, and the difference map.
W, H = A.size
side = Image.new('RGB', (W * 2 + 24, H), (10, 10, 10))
side.paste(A, (0, 0)); side.paste(B, (W + 24, 0))
side.save(os.path.join(out, f'{name}-side-by-side.png'))
heat = np.clip(d.astype(np.int32) * 4, 0, 255).astype(np.uint8)
Image.fromarray(heat).save(os.path.join(out, f'{name}-difference.png'))
print('wrote', os.path.join(out, f'{name}-side-by-side.png'))
