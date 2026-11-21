# Mercury's desktop window, native

A trial of drawing Mercury's window with [Rune](https://github.com/anomalyco/rune) (Rust, GPU, no
browser inside) instead of a web page in Electron. So far it is one screen, Home, with invented
tasks, built to answer one question: does the look survive the move.

It does. Against the real renderer at 1280 by 760, 2x, dark:

| | mean difference | similarity |
|---|---|---|
| whole window | 0.88 of 255 | 0.995 |
| sky, rail, top strip | 0.00 to 0.17 | 0.999 to 1.000 |
| cards and composer | 1.9 to 2.1 | 0.990 to 0.995 |

Text lands where the browser puts it to a hundredth of a point ("Opus 5.5" is 53.46 points wide in
both).

## Run it

```sh
cd desktop
cargo run                                                  # the window
cargo run -- --snap shots/home.png --size 1280x760 --scale 2   # a picture, no window
cargo run -- --tree shots/tree.json --size 1280x760 --scale 2  # every element's box and colour
```

The first build fetches Rune and takes about a minute. `rust-toolchain.toml` names the toolchain
Rune pins, and rustup installs it by itself.

## Compare it with the real one

```sh
arch -arm64 node desktop/tools/reference.mjs <checkout with renderer/dist> /tmp/ref dark 1280 760
python3 desktop/tools/compare.py /tmp/ref/mercury-real-inbox-dark.png desktop/shots/home.png /tmp/ref
```

`reference.mjs` photographs the built renderer with an invented inbox and writes every element's
measurements; the screen here was written from those numbers, not by eye. `compare.py` needs
`pillow` and `numpy`, prints the difference per region and writes a side-by-side and a difference map.

## What still differs, and why

- **Glass is less saturated.** The web app's glass is `blur(20px) saturate(1.5)`; Rune's
  `backdrop_blur` has no saturation. Up to 6 of 255 warmer in the real one, inside the composer.
- **Four blurred surfaces a frame.** Rune's `MAX_BACKDROP_BLURS` is 4 and Home has five, so the two
  small doors go unblurred. With the constant raised to 8 in a local copy all five blur and the
  doors go from 0.912 to 0.989. The code here turns their blur on by itself when Rune allows it.
- **A cut-off title loses its last space.** The browser keeps the space before the ellipsis.
- **Large text sits a fraction of a pixel differently**, which shows only in a difference map.

## What is where

- `src/main.rs`: the screen.
- `src/sky.rs`: the dithered sky, ported from `renderer/src/home/sky.ts` with the same seed, so it
  is the same picture.
- `src/icons.rs`: the icons, with the web app's own paths and strokes.
- `assets/fonts`: Geist and Geist Mono (SIL Open Font License), the files the web app ships.
