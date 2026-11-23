"""Drive the native window through a table scene's steps and measure it against the web's picture.

    python3 tools/table.py <shots> <scene> [width height]

A table scene (`tools/scenes/table*.mjs`) photographs the web window after pressing, typing and
keying into a sheet, and leaves in its fixture the same steps in the native window's script
(`tableScript`) and what the page would have written (`wrote`). This runs the native window on
that fixture with that script (`--script`, `--snap --at` once the last step has settled), and
prints, of 255, the difference whole and in the sheet's card, and whether what the native window
would have written (`MERCURY_WROTE`) is the web's, byte for byte.
"""
import json, os, subprocess, sys

import numpy as np
from PIL import Image

shots, scene = sys.argv[1], sys.argv[2]
w, h = (int(sys.argv[3]), int(sys.argv[4])) if len(sys.argv) > 4 else (1280, 760)
base = os.path.join(shots, scene)
fixture = json.load(open(base + '-fixture.json'))
script = fixture.get('tableScript', [])
json.dump(script, open(base + '-script.json', 'w'))
wrote = base + '-native-wrote.json'
if os.path.exists(wrote):
    os.remove(wrote)
# Under the half second after which a headless window draws Rune's stand-in for a title, which
# the web window's picture never shows (the system draws its titles).
at = fixture.get('tableEnd', 0.5) + 0.1
view = ['--thread'] if 'docOpen' in fixture else []
env = dict(os.environ, MERCURY_WROTE=wrote)
subprocess.run(['./target/debug/mercury', '--fixture', base + '-fixture.json', *view, '--script', base + '-script.json',
                '--snap', base + '.png', '--size', f'{w}x{h}', '--scale', '2', '--at', f'{at:.3f}'], check=True, capture_output=True, env=env)

a = np.asarray(Image.open(base + '-real.png').convert('RGB')).astype(float)
b = np.asarray(Image.open(base + '.png').convert('RGB')).astype(float)
d = np.abs(a - b)
# The sheet's card beside the conversation: the right part of the body, its head and bar included.
card = d[232:1446, 990:2482] if (w, h) == (1280, 760) else d
print(f'{scene}: whole {d.mean():.2f}  sheet {card.mean():.2f}')
web = fixture.get('wrote', [])
native = json.load(open(wrote)) if os.path.exists(wrote) else []
if web or native:
    same = [x.get('text') for x in web] == [x.get('text') for x in native]
    print(f'  wrote: web {len(web)}, native {len(native)}, the same words: {same}')
    if not same:
        for x, y in zip(web, native):
            if x.get('text') != y.get('text'):
                print('  web   :', repr(x.get('text'))[:400])
                print('  native:', repr(y.get('text'))[:400])
                break
