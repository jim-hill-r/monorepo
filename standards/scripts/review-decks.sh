#!/usr/bin/env bash
# Build every review deck in docs/reviews/*/deck.md to HTML and PDF with the
# Marp CLI, into build/reviews/<folder>/ (deck.html, deck.pdf and the deck's
# images). See docs/reviews/README.md.
#
#   review-decks.sh [OUT_DIR]
#
# Needs Node (npx) and a Chrome or Chromium for the PDF; set CHROME_PATH if
# Marp can't find one.
#
# REVIEW_DEMOS names the deck folders whose demo videos to record first
# (space-separated, or `all`), from each deck's demos/*.json into
# OUT_DIR/<folder>/demos/ with docs/reviews/demo-recorder. Unset, no demos
# are recorded: a demo shows the live app, so it is recorded only for the
# deck being written, not again for older decks. Recording needs ffmpeg and
# the recorder's setup in docs/reviews/demo-recorder/README.md.
set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

marp_version="4.5.1"
out="${1:-build/reviews}"

shopt -s nullglob
decks=(docs/reviews/*/deck.md)
if [ "${#decks[@]}" -eq 0 ]; then
  echo "No review decks found."
  exit 0
fi

rm -rf "$out"
mkdir -p "$out"
for deck in "${decks[@]}"; do
  folder="$(basename "$(dirname "$deck")")"
  cp -r "docs/reviews/$folder" "$out/$folder"
  demos=("docs/reviews/$folder"/demos/*.json)
  if [ "${#demos[@]}" -gt 0 ] && { [ "${REVIEW_DEMOS:-}" = all ] || [[ " ${REVIEW_DEMOS:-} " == *" $folder "* ]]; }; then
    node docs/reviews/demo-recorder/src/record.ts --out "$out/$folder/demos" "${demos[@]}"
  fi
  for format in html pdf; do
    npx --yes "@marp-team/marp-cli@$marp_version" --no-stdin --html --allow-local-files \
      "--$format" "$out/$folder/deck.md" -o "$out/$folder/deck.$format"
  done
  rm "$out/$folder/deck.md"
  echo "Built $deck"
done
