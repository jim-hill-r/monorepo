#!/usr/bin/env bash
# DS-*: projects that have adopted the design system use its tokens, not raw
# colours (design_system/docs/plans/2026-11-design-system.md, "Using the tokens").
# Run by //:audit. Reads only repo files. Exits non-zero on any error.
#
#   DS-001  A CSS file in an adopted project has a raw colour in a declaration:
#           a hex colour (#rgb to #rrggbbaa) or a colour function (rgb(),
#           rgba(), hsl(), hsla(), hwb(), lab(), lch(), oklab(), oklch()).
#           Use a --ds- token, or an app's own property defined from one.
#   DS-002  A Rust file in an adopted project builds a colour from literals
#           (Bevy's Color::srgb(0.1, ...), Color::hex("..."), Srgba::rgb(...)
#           and friends). Use design_system_palette instead.
#
# A directory has adopted the design system when it holds a Cargo.toml that
# depends on a design_system crate, a design/theme.toml (a custom app theme),
# or a mise.toml with an `# affected-by:` line naming design_system (how a
# CSS-only app, which links dist/ without a crate, says so). Every tracked
# file under it is checked; design_system/ itself and the generated
# */design/theme.css are not.
#
# A line that really needs a literal colour (a test of the contrast maths, a
# brand mark that must be exact) ends with the comment `ds-allow-raw-colour`
# and a reason.
set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

errors=0
err() { echo "error[$1] $2"; errors=$((errors + 1)); }

# The adopted directories, one per line.
adopted=$(
  {
    # grep -l exits 1 when a batch has no match, so xargs does too.
    git ls-files -- '*Cargo.toml' | xargs -r grep -l 'design_system' || true
    git ls-files -- '*/design/theme.toml' | xargs -r -n1 dirname || true
    git ls-files -- '*mise.toml' | xargs -r grep -lE '^#[[:space:]]*affected-by:.*design_system' || true
  } | xargs -r -n1 dirname | grep -v -e '^design_system$' -e '^design_system/' -e '^\.$' | sort -u
)

# in_adopted FILE: whether FILE is under one of the adopted directories.
in_adopted() {
  local d
  while IFS= read -r d; do
    [ -n "$d" ] || continue
    case "$1" in "$d"/*) return 0 ;; esac
  done <<<"$adopted"
  return 1
}

hex='#[0-9a-fA-F]{3}([0-9a-fA-F]{1,5})?([^0-9a-zA-Z_-]|$)'
fn='(^|[^a-zA-Z0-9_-])(rgba?|hsla?|hwb|lab|lch|oklab|oklch)\('

while IFS= read -r f; do
  [ -n "$f" ] || continue
  case "$f" in design_system/* | */design/theme.css) continue ;; esac
  in_adopted "$f" || continue
  # Blank out comments (keeping line numbers), then look for a colour in a
  # declaration: after a colon, on a line that doesn't open a rule.
  while IFS=: read -r n line; do
    err DS-001 "$f:$n uses a raw colour; use a --ds- token: $(echo "$line" | sed 's/^[[:space:]]*//')"
  done < <(
    awk '
      {
        out = ""; s = $0
        while (length(s) > 0) {
          if (inc) {
            i = index(s, "*/"); if (i == 0) { s = ""; break }
            s = substr(s, i + 2); inc = 0
          } else {
            i = index(s, "/*"); if (i == 0) { out = out s; s = ""; break }
            out = out substr(s, 1, i - 1); s = substr(s, i + 2); inc = 1
          }
        }
        if ($0 ~ /ds-allow-raw-colour/) out = ""
        print NR ":" out
      }
    ' "$f" | grep -E "^[0-9]+:.*:[^{};]*($hex|$fn)" | grep -vE '\{[[:space:]]*$' || true
  )
done < <(git ls-files -- '*.css')

ctor='(Color|Srgba|LinearRgba|Hsla|Hsva|Hwba|Laba|Lcha|Oklaba|Oklcha)::(srgba?|srgba?_u8|rgba?|rgba?_u8|rgb_linear|linear_rgba?|hsla?|hsva?|hwba?|laba?|lcha?|oklaba?|oklcha?|hex|new)\([[:space:]]*[-0-9."]'
while IFS= read -r f; do
  [ -n "$f" ] || continue
  case "$f" in design_system/*) continue ;; esac
  in_adopted "$f" || continue
  while IFS=: read -r n line; do
    err DS-002 "$f:$n builds a colour from literals; use design_system_palette: $(echo "$line" | sed 's/^[[:space:]]*//')"
  done < <(grep -nE "$ctor" "$f" | grep -v 'ds-allow-raw-colour' || true)
done < <(git ls-files -- '*.rs')

[ "$errors" -eq 0 ]
