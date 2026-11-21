#!/bin/sh
# Build the native window and put it in ~/Applications as "Mercury Native", next to Mercury.
#
#   desktop/tools/install-mac.sh            install, and open it only if it was already open
#   desktop/tools/install-mac.sh --open     install and open it
#
# The bundle is made and swapped in by Rune's own installer (scripts/mac-app in the Rune
# revision Cargo.toml pins), which signs it and exchanges it with the old one in one step, so
# a running copy is replaced whole or not at all. Opening a window takes the keyboard, which is
# why a plain install does not: an agent can run it while the person is typing somewhere else.
set -eu
here=$(cd "$(dirname "$0")/.." && pwd)
cd "$here"

cargo build --release

# Where cargo checked Rune out: its installer lives beside the crates.
rune=$(cargo metadata --format-version 1 2>/dev/null | python3 -c '
import json, sys
for p in json.load(sys.stdin)["packages"]:
    if p["name"] == "rune":
        print(p["manifest_path"].rsplit("/crates/rune/", 1)[0])
        break
')
[ -x "$rune/scripts/mac-app" ] || { echo "Rune's installer was not found under $rune" >&2; exit 1; }

only="--only-relaunch"
[ "${1:-}" = "--open" ] && only=""
# shellcheck disable=SC2086
exec "$rune/scripts/mac-app" --name "Mercury Native" --id io.github.matheusmercadante.mercury.native \
  --exe mercury --bin target/release/mercury --icon "$here/../build/icon.icns" $only
