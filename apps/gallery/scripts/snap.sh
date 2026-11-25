#!/bin/bash
# Headless pictures of the gallery: no window, no focus. The same renderer as the window on a fixed virtual clock, so a picture is
# a fact about the components, not a screenshot of whatever the Mac was doing. (Rune's own `--snap` tooling, as Neko's
# `apps/neko-rune/scripts/snap.sh` uses it.)
#
#   apps/gallery/scripts/snap.sh [out-dir] [scene ...]     (default /tmp/rune-ui, every scene)
#
# RUNE_UI_WIDTH (1280) and RUNE_UI_SCALE (2) change the picture; the height follows the content.
# Writes <scene>-light.png and <scene>-dark.png, and <scene>-light.tree.json (the element tree, for tests and reviews).
set -euo pipefail
here="$(cd "$(dirname "$0")/.." && pwd)"
out="${1:-/tmp/rune-ui}"
shift || true
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$HOME/Library/Caches/neko/build-rune-ui}"
cd "$here"
cargo build --quiet -p rune-ui-gallery
bin="$CARGO_TARGET_DIR/debug/rune-ui-gallery"
mkdir -p "$out"
scenes=("$@")
if [ ${#scenes[@]} -eq 0 ]; then
  scenes=($("$bin" --list-scenes))
fi
size="${RUNE_UI_WIDTH:-1280}x900"
for scene in "${scenes[@]}"; do
  for theme in light dark; do
    "$bin" --snap "$out/$scene-$theme.png" --scene "$scene" --theme "$theme" --size "$size" --scale "${RUNE_UI_SCALE:-2}" >/dev/null 2>&1
  done
  "$bin" --tree "$out/$scene-light.tree.json" --scene "$scene" --theme light --size "$size" >/dev/null 2>&1
  echo "$scene"
done
echo "wrote $out"
