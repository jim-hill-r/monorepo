#!/usr/bin/env bash
# Tests standards/scripts/design-check.sh (DS-001, DS-002) in throwaway repos:
# token-only apps pass, and raw colours fail only in adopted projects.
#   bash standards/tests/design-check.test.sh
set -uo pipefail

root=$(cd "$(dirname "$0")/../.." && pwd)
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
failures=0

# new_repo NAME: a repo with the design system, one Dioxus app that depends on
# it (app/web), a CSS-only app that says so in its mise.toml (site), a game
# (game), an app with a custom theme (brand) and one that hasn't adopted it
# (legacy). Every file uses tokens only.
new_repo() {
  local repo=$work/$1
  mkdir -p "$repo/standards/scripts" "$repo/design_system/dist" "$repo/app/web/assets" \
    "$repo/site/static" "$repo/game/src" "$repo/brand/design" "$repo/brand/web" "$repo/legacy"
  cp "$root/standards/scripts/design-check.sh" "$repo/standards/scripts/"
  echo ':root { --ds-color-bg: #ffffff; }' >"$repo/design_system/dist/tokens.css"
  cat >"$repo/app/web/Cargo.toml" <<'TOML'
[dependencies]
design_system_dioxus = { path = "../../design_system/dioxus" }
TOML
  cat >"$repo/app/web/assets/main.css" <<'CSS'
/* Colours come from the tokens, never #fff or rgb(0 0 0). */
.card {
  background: var(--ds-color-surface);
  color: var(--ds-color-text);
}
#fab { color: var(--ds-color-accent); }
a:hover #bad {
  color: var(--ds-color-link);
}
CSS
  printf '# affected-by: ../design_system\n' >"$repo/site/mise.toml"
  echo 'body { background: var(--ds-color-bg); }' >"$repo/site/static/site.css"
  cat >"$repo/game/Cargo.toml" <<'TOML'
[dependencies]
design_system_palette = { path = "../design_system/palette" }
TOML
  cat >"$repo/game/src/main.rs" <<'RS'
fn color(c: Rgba) -> Color {
    Color::srgba_u8(c.0, c.1, c.2, c.3)
}
RS
  echo 'extends = "colorful"' >"$repo/brand/design/theme.toml"
  echo '[data-theme="bears"] { --ds-color-bg: #0b162a; }' >"$repo/brand/design/theme.css"
  echo '.b { color: var(--ds-color-text); }' >"$repo/brand/web/style.css"
  echo '.old { color: #8b4513; }' >"$repo/legacy/style.css"
  echo 'fn f() { Color::srgb(0.1, 0.2, 0.3); }' >"$repo/legacy/main.rs"
  git -C "$repo" init -q
  echo "$repo"
}

add_all() { git -C "$1" add -A; }

# expect NAME REPO WANT_STATUS [WANT_TEXT]
expect() {
  local name=$1 repo=$2 want=$3 text=${4:-}
  out=$(cd "$repo" && bash standards/scripts/design-check.sh 2>&1)
  local got=$?
  if [ "$got" -ne "$want" ] || { [ -n "$text" ] && [[ "$out" != *"$text"* ]]; }; then
    echo "FAIL $name: exit $got (want $want), want text '$text' in:"
    echo "$out" | sed 's/^/    /'
    failures=$((failures + 1))
  fi
}

r=$(new_repo ok); add_all "$r"
expect "tokens only, raw colours only outside adopted projects" "$r" 0

r=$(new_repo hex); echo '.x { border-color: #c83803; }' >>"$r/app/web/assets/main.css"; add_all "$r"
expect "hex in a one-line rule" "$r" 1 "error[DS-001] app/web/assets/main.css:10 uses a raw colour"

r=$(new_repo fn); printf 'h1 {\n  color: rgb(10 20 30);\n}\n' >>"$r/site/static/site.css"; add_all "$r"
expect "rgb() in a CSS-only app" "$r" 1 "error[DS-001] site/static/site.css:3 uses a raw colour"

r=$(new_repo mix); echo '.m { background: color-mix(in srgb, var(--ds-color-bg), #000 10%); }' >>"$r/brand/web/style.css"; add_all "$r"
expect "hex inside color-mix() in an app with a custom theme" "$r" 1 "error[DS-001] brand/web/style.css:2"

r=$(new_repo oklch); echo '.o { outline-color: oklch(70% 0.1 200); }' >>"$r/app/web/assets/main.css"; add_all "$r"
expect "oklch()" "$r" 1 "error[DS-001]"

r=$(new_repo allow); echo '.logo { fill: #c83803; } /* ds-allow-raw-colour: exact brand mark */' >>"$r/app/web/assets/main.css"; add_all "$r"
expect "allowed with a comment" "$r" 0

r=$(new_repo rust); echo 'const RED: Color = Color::srgb(0.8, 0.1, 0.1);' >>"$r/game/src/main.rs"; add_all "$r"
expect "Bevy colour from literals" "$r" 1 "error[DS-002] game/src/main.rs:4 builds a colour from literals"

r=$(new_repo rusthex); echo 'let c = Srgba::hex("#ff0000").unwrap();' >>"$r/game/src/main.rs"; add_all "$r"
expect "Srgba::hex" "$r" 1 "error[DS-002]"

r=$(new_repo untracked); echo '.x { color: #fff; }' >"$r/app/web/assets/new.css"
git -C "$r" add -A ':!app/web/assets/new.css'
expect "untracked files are not checked" "$r" 0

if [ "$failures" -gt 0 ]; then
  echo "design-check.test.sh: $failures failure(s)"
  exit 1
fi
echo "design-check.test.sh: ok"
