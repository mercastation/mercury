"""Draw a scene with the pointer resting where the web window's picture had it, and say how far
the control under it is from the web window's.

  python3 tools/hover.py <shots dir> <scene> [<scene> ...]

A scene that rests the pointer (`tools/scenes/pointer.mjs`) leaves in its fixture where
(`pointer`) and for how long (`rested`, in milliseconds). This moves the native window's pointer
there the same way, in two steps 30 ms apart (a `--script`), draws it that long after the second
(`--snap --at`), and measures, of 255, the control the pointer is on: the element under it in the
web window's dump and every element around it that wears the same pointer, the row a title is
in, say. Run from `desktop/`, after `cargo build`.
"""
import json, os, subprocess, sys

import numpy as np
from PIL import Image

shots, scenes = sys.argv[1], sys.argv[2:]


def view_of(scene):
    """The command line a scene's picture is drawn with, by its name, as the notes draw them."""
    if scene.startswith('thread'):
        return ['--thread']
    for tab in ('later', 'progress', 'done'):
        if scene.startswith(f'inbox-{tab}'):
            return ['--view', 'inbox', '--tab', tab]
    if scene.startswith(('inbox', 'compose-card', 'palette')):
        return ['--view', 'inbox']
    return []


def control(dump, x, y):
    """The element under the point, and the outermost one around it wearing the same pointer."""
    nodes = dump['nodes']
    by_id = {n['id']: n for n in nodes}
    inside = lambda n: n['rect'][0] <= x <= n['rect'][0] + n['rect'][2] and n['rect'][1] <= y <= n['rect'][1] + n['rect'][3]
    shown = lambda n: n['style'].get('visibility') != 'hidden' and n['style'].get('pointerEvents') != 'none'
    under = [n for n in nodes if inside(n) and shown(n)]
    if not under:
        return None, None
    hit = under[-1]
    top = hit
    while True:
        parent = by_id.get(top['parent'])
        if not parent or parent['tag'] == 'body' or parent['style'].get('cursor') != hit['style'].get('cursor') or not inside(parent):
            return hit, top
        top = parent


def mean(a, b, box, scale=2):
    x, y, w, h = [int(round(v * scale)) for v in box]
    x, y = max(x, 0), max(y, 0)
    d = np.abs(a[y:y + h, x:x + w].astype(np.int16) - b[y:y + h, x:x + w].astype(np.int16))
    return float(d.mean()) if d.size else float('nan')


for scene in scenes:
    base = os.path.join(shots, scene)
    fixture = json.load(open(base + '-fixture.json'))
    dump = json.load(open(base + '-dump.json'))
    x, y = fixture['pointer']
    rested = fixture.get('rested', 200) / 1000
    script = [{'t': 0.05, 'type': 'move', 'x': x - 8, 'y': y - 4}, {'t': 0.08, 'type': 'move', 'x': x, 'y': y}]
    json.dump(script, open(base + '-script.json', 'w'))
    w, h = dump['viewport']
    native = base + '.png'
    subprocess.run(['./target/debug/mercury', '--fixture', base + '-fixture.json', *view_of(scene), '--script', base + '-script.json',
                    '--snap', native, '--size', f'{w}x{h}', '--scale', '2', '--at', f'{0.08 + rested:.3f}'], check=True, capture_output=True)
    a = np.asarray(Image.open(base + '-real.png').convert('RGB'))
    b = np.asarray(Image.open(native).convert('RGB'))
    hit, top = control(dump, x, y)
    web = next((p['cursor'] for p in dump.get('pointers', []) if p.get('resting')), '?')
    if top is None:
        print(f'{scene}: nothing under {x},{y}')
        continue
    box = top['rect']
    grown = [box[0] - 4, box[1] - 4, box[2] + 8, box[3] + 8]
    name = f"{top['tag']}.{'.'.join((top.get('cls') or '').split()[:2])}"
    print(f"{scene:26} {name:34.34} [{box[0]:7.1f} {box[1]:6.1f} {box[2]:6.1f} {box[3]:5.1f}]  control {mean(a, b, box):5.2f}  with 4 around {mean(a, b, grown):5.2f}  whole {float(np.abs(a.astype(np.int16) - b.astype(np.int16)).mean()):5.2f}  web pointer {web}")
