#!/usr/bin/env bash
# Tests standards/scripts/review-decks.sh, which builds docs/reviews/*/deck.md
# to HTML and PDF (npx is a stub that writes the -o file): each deck lands in
# OUT_DIR/<folder>/ with deck.html, deck.pdf and its images but not deck.md,
# an absolute OUT_DIR outside the repo works (pane's release passes one), a
# stale deck is cleared, and a repo with no decks builds nothing.
# Run from anywhere:
#   bash standards/tests/review-decks.test.sh
set -uo pipefail

script="$(cd "$(dirname "$0")/.." && pwd)/scripts/review-decks.sh"
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

repo="$work/repo"
mkdir -p "$repo/standards/scripts" "$repo/docs/reviews/2026-11-25-cookbook" "$work/stubs"
cp "$script" "$repo/standards/scripts/review-decks.sh"
printf '# Cookbook\n' >"$repo/docs/reviews/2026-11-25-cookbook/deck.md"
printf 'png' >"$repo/docs/reviews/2026-11-25-cookbook/home.png"
printf '# Reviews\n' >"$repo/docs/reviews/README.md"
git -C "$repo" init -q

# npx stub: logs its arguments and writes the file named after -o.
cat >"$work/stubs/npx" <<SH
#!/bin/sh
echo "npx \$*" >>"$work/calls"
while [ \$# -gt 0 ]; do
  if [ "\$1" = "-o" ]; then echo built >"\$2"; fi
  shift
done
SH
chmod +x "$work/stubs/npx"

failures=0
fail() { echo "FAIL $1"; [ -z "${2:-}" ] || echo "$2" | sed 's/^/    /'; failures=$((failures + 1)); }

# decks DIR ARGS...: runs the script from DIR with the stub npx.
decks() {
  local dir=$1
  shift
  : >"$work/calls"
  out=$(cd "$dir" && PATH="$work/stubs:$PATH" bash standards/scripts/review-decks.sh "$@" 2>&1)
  status=$?
}

# An absolute OUT_DIR outside the repo, with a stale deck in it.
dest="$work/site/public/reviews"
mkdir -p "$dest/2026-01-01-old"
decks "$repo" "$dest"
[ "$status" -eq 0 ] || fail "build: exit $status" "$out"
for f in deck.html deck.pdf home.png; do
  [ -f "$dest/2026-11-25-cookbook/$f" ] || fail "build: missing $f" "$(ls -R "$dest")"
done
[ ! -e "$dest/2026-11-25-cookbook/deck.md" ] || fail "build: deck.md left in the output"
[ ! -e "$dest/2026-01-01-old" ] || fail "build: stale deck kept"
[ ! -e "$dest/README.md" ] || fail "build: README.md copied"
grep -q -- "--pdf" "$work/calls" || fail "build: no PDF run" "$(cat "$work/calls")"

# No decks: says so and leaves the output alone.
rm -rf "$repo/docs/reviews/2026-11-25-cookbook"
decks "$repo" "$dest"
[ "$status" -eq 0 ] || fail "no-decks: exit $status" "$out"
case "$out" in *"No review decks found."*) ;; *) fail "no-decks: message" "$out" ;; esac
[ ! -s "$work/calls" ] || fail "no-decks: npx ran" "$(cat "$work/calls")"

if [ "$failures" -gt 0 ]; then
  echo "$failures failure(s)"
  exit 1
fi
echo "review-decks.sh: all tests passed"
