#!/usr/bin/env bash
# Convierte los cuadros de docs/frames/* en GIFs (necesita ffmpeg).
# Uso: cargo run --release -- capture   y luego   bash tools/make_gifs.sh
set -e
cd "$(dirname "$0")/.."
for dir in docs/frames/*/; do
  name=$(basename "$dir")
  fps=$(cat "$dir/fps.txt" 2>/dev/null || echo 20)
  ffmpeg -loglevel error -y -framerate "$fps" -i "$dir/%03d.png" \
    -vf "split[a][b];[a]palettegen=max_colors=192[p];[b][p]paletteuse=dither=bayer:bayer_scale=4" \
    -loop 0 "docs/$name.gif"
  echo "docs/$name.gif"
done
