#!/usr/bin/env bash
# Build every review deck in docs/reviews/*/deck.md to HTML and PDF with the
# Marp CLI, into build/reviews/<folder>/ (deck.html, deck.pdf and the deck's
# images). See docs/reviews/README.md.
#
#   review-decks.sh [OUT_DIR]
#
# Needs Node (npx) and a Chrome or Chromium for the PDF; set CHROME_PATH if
# Marp can't find one.
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
  for format in html pdf; do
    npx --yes "@marp-team/marp-cli@$marp_version" --html --allow-local-files \
      "--$format" "$out/$folder/deck.md" -o "$out/$folder/deck.$format"
  done
  rm "$out/$folder/deck.md"
  echo "Built $deck"
done
