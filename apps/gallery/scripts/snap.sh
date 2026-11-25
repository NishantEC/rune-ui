#!/bin/bash
# Headless pictures of the gallery: no window, no focus. The same renderer as the window on a fixed virtual clock, so a picture is a
# fact about the components, not a screenshot of whatever the Mac was doing (Rune's own `--snap` tooling).
#
#   apps/gallery/scripts/snap.sh [out-dir] [scene ...]     (default /tmp/rune-ui; every group, materials and the open overlays)
#   RUNE_UI_ITEMS="button switch" apps/gallery/scripts/snap.sh   also pictures single items (item-<name>-light.png ...)
#
# The canvas is 8000 points tall (Metal textures stop at 16384 pixels) and cropped to the page element, so each picture is as tall as its content. RUNE_UI_SCALE (2) sets the density.
set -euo pipefail
here="$(cd "$(dirname "$0")/.." && pwd)"
out="${1:-/tmp/rune-ui}"
shift || true
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$HOME/Library/Caches/neko/build-rune-ui}"
cd "$here"
cargo build --quiet -p gallery
bin="$CARGO_TARGET_DIR/debug/gallery"
mkdir -p "$out"
scenes=("$@")
if [ ${#scenes[@]} -eq 0 ]; then
  scenes=($("$bin" --list-scenes | grep -v '^all$'))
fi
snap() { # name theme [env]
  "$bin" --snap "$out/$1-$2.png" --scene "${3:-$1}" --theme "$2" --size 1320x8000 --scale "${RUNE_UI_SCALE:-2}" --crop canvas >/dev/null 2>&1
}
for scene in "${scenes[@]}"; do
  for theme in light dark; do snap "$scene" "$theme"; done
  echo "$scene"
done
for item in ${RUNE_UI_ITEMS:-}; do
  for theme in light dark; do
    RUNE_UI_ITEM="$item" "$bin" --snap "$out/item-$item-$theme.png" --scene actions --theme "$theme" --size 1320x8000 --scale "${RUNE_UI_SCALE:-2}" --crop canvas >/dev/null 2>&1
  done
  echo "item-$item"
done
"$bin" --tree "$out/actions-light.tree.json" --scene actions --theme light --size 1320x4000 >/dev/null 2>&1
echo "wrote $out"
