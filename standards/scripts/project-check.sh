#!/usr/bin/env bash
# NAM-002, NAM-004, DOC-* and LIN-*: the per-project checks the removed
# `standards` CLI ran, ported to the audit. Run by //:audit. Reads only repo
# files.
#
#   NAM-002  A project's package name (Cargo.toml [package] name, or
#            package.json "name") matches its directory name, or the parent
#            directory and the directory joined with "_" (cahokia/web ->
#            cahokia_web). standards/docs/naming.md.
#   NAM-004  A proof of concept project (its package name or path says poc
#            or proof) is named poc_<name>. standards/docs/naming.md.
#   DOC-001  A project has a README.md.
#   DOC-002  Its README.md has a heading with the project's name: its package
#            name, its directory name or its path (bear_down/cloudflare), or
#            the name with spaces for "_" and "-", optionally after "The"
#            (psy_ents -> Psy Ents or The Psy Ents).
#   DOC-003  A project has a CONTRIBUTING.md.
#   DOC-004  Its CONTRIBUTING.md has a "Getting Started" heading.
#            standards/docs/documentation.md.
#   LIN-001  Every Rust package sets clippy's unwrap_used and expect_used to
#            "warn" (or stricter), in [lints.clippy] or through
#            `[lints] workspace = true` and [workspace.lints.clippy].
#   LIN-002  Every Rust package forbids unsafe_code, in [lints.rust] or
#            through its workspace. "deny" passes when a comment on the line
#            above says why the package needs its one allowed unsafe item.
#            standards/docs/rust.md.
#
# A project is a directory with a mise.toml (not the repo root). Every check
# is an error. Projects under the "# On Hold Projects" list in standards/docs/issues/README.md
# are exempt from DOC-* and LIN-* until they come back off hold. Exits
# non-zero on any error.
set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

errors=0
err() { echo "error[$1] $2"; errors=$((errors + 1)); }

# package_name DIR: the Cargo.toml [package] name, else the package.json name,
# else nothing.
package_name() {
  local d=$1 name=""
  if [ -f "$d/Cargo.toml" ]; then
    name=$(awk '
      /^\[/ { in_pkg = ($0 ~ /^\[package\][[:space:]]*$/) ; next }
      in_pkg && /^[[:space:]]*name[[:space:]]*=/ {
        sub(/^[^=]*=[[:space:]]*"/, ""); sub(/".*/, ""); print; exit
      }' "$d/Cargo.toml")
  fi
  if [ -z "$name" ] && [ -f "$d/package.json" ]; then
    name=$(sed -nE 's/^  "name"[[:space:]]*:[[:space:]]*"([^"]*)".*/\1/p' "$d/package.json" | head -n1)
  fi
  printf '%s' "$name"
}

# has_heading FILE TEXT: FILE has a Markdown heading whose text is TEXT,
# ignoring case.
has_heading() {
  awk -v want="$2" '
    BEGIN { want = tolower(want) }
    /^[[:space:]]*#/ {
      h = $0; sub(/^[[:space:]]*#+[[:space:]]*/, "", h); sub(/[[:space:]]+$/, "", h)
      if (tolower(h) == want) { found = 1; exit }
    }
    END { exit !found }' "$1"
}

# lints FILE: one "section key level" line per lint in FILE's lint tables,
# plus "lints workspace true" when it inherits its workspace's lints. A level
# may be a string or a table with level = "...".
lints() {
  awk '
    /^[[:space:]]*\[/ {
      s = $0; gsub(/[[:space:]\[\]]/, "", s)
      sec = (s == "lints" || s == "lints.clippy" || s == "lints.rust" ||
             s == "workspace.lints.clippy" || s == "workspace.lints.rust") ? s : ""
      next
    }
    sec != "" && /=/ {
      key = $0; sub(/[[:space:]]*=.*/, "", key); gsub(/[[:space:]]/, "", key)
      val = $0; sub(/^[^=]*=[[:space:]]*/, "", val)
      if (val ~ /level[[:space:]]*=/) { sub(/.*level[[:space:]]*=[[:space:]]*/, "", val) }
      gsub(/"/, "", val); sub(/[[:space:],}#].*/, "", val)
      print sec, key, val
    }' "$1"
}

# workspace_of FILE: FILE itself when it has [workspace], else the nearest
# Cargo.toml above it that has one.
workspace_of() {
  local d
  if grep -q '^\[workspace\]' "$1"; then echo "$1"; return; fi
  d=$(dirname "$(dirname "$1")")
  while [ "$d" != "." ] && [ "$d" != "/" ]; do
    if [ -f "$d/Cargo.toml" ] && grep -q '^\[workspace\]' "$d/Cargo.toml"; then
      echo "$d/Cargo.toml"
      return
    fi
    d=$(dirname "$d")
  done
  if [ -f Cargo.toml ] && grep -q '^\[workspace\]' Cargo.toml; then echo Cargo.toml; fi
}

# level_ok LEVEL MIN: LEVEL is MIN or stricter (allow < warn < deny < forbid).
level_ok() {
  local rank_allow=0 rank_warn=1 rank_deny=2 rank_forbid=3 have want
  have="rank_$1"; want="rank_$2"
  [ -n "${!have:-}" ] && [ "${!have}" -ge "${!want}" ]
}

projects=$(git ls-files -- '*/mise.toml' | xargs -r -n1 dirname | sort -u)

# The on hold projects: the "- name" lines under "# On Hold Projects".
on_hold=""
if [ -f standards/docs/issues/README.md ]; then
  on_hold=$(awk '
    /^#/ { in_list = ($0 ~ /^#[[:space:]]+On Hold Projects[[:space:]]*$/); next }
    in_list && /^[[:space:]]*-[[:space:]]+/ { sub(/^[[:space:]]*-[[:space:]]+/, ""); sub(/[[:space:]]+$/, ""); print }
  ' standards/docs/issues/README.md)
fi
# held PATH: PATH is in, or is, an on hold project.
held() {
  local p
  for p in $on_hold; do
    case "$1" in "$p" | "$p"/*) return 0 ;; esac
  done
  return 1
}

for d in $projects; do
  name=$(package_name "$d")
  base=$(basename "$d")
  parent=$(basename "$(dirname "$d")")
  if [ -n "$name" ] && [ "$name" != "$base" ] && [ "$name" != "${parent}_$base" ]; then
    err NAM-002 "$d: package name '$name' must be '$base' or '${parent}_$base'"
  fi
  lower=$(printf '%s %s' "$name" "$d" | tr '[:upper:]' '[:lower:]')
  if [[ "$lower" =~ (^|[/_ -])(poc|proof)([/_ -]|$) ]] && [[ "${name:-$base}" != poc_* ]]; then
    err NAM-004 "$d: proof of concept project '${name:-$base}' must be named poc_<name>"
  fi

  held "$d" && continue
  if [ ! -f "$d/README.md" ]; then
    err DOC-001 "$d has no README.md"
  elif ! has_heading "$d/README.md" "${name:-$base}" && ! has_heading "$d/README.md" "$base" &&
    ! has_heading "$d/README.md" "$d" && ! has_heading "$d/README.md" "$(tr '_-' '  ' <<<"${name:-$base}")" &&
    ! has_heading "$d/README.md" "The $(tr '_-' '  ' <<<"${name:-$base}")"; then
    err DOC-002 "$d/README.md has no '# ${name:-$base}' heading"
  fi
  if [ ! -f "$d/CONTRIBUTING.md" ]; then
    err DOC-003 "$d has no CONTRIBUTING.md"
  elif ! has_heading "$d/CONTRIBUTING.md" "Getting Started"; then
    err DOC-004 "$d/CONTRIBUTING.md has no 'Getting Started' heading"
  fi
done

for f in $(git ls-files -- 'Cargo.toml' '*/Cargo.toml'); do
  grep -q '^\[package\]' "$f" || continue
  held "$f" && continue
  own=$(lints "$f")
  sec_clippy=lints.clippy sec_rust=lints.rust src=$f where=""
  if grep -qx 'lints workspace true' <<<"$own"; then
    src=$(workspace_of "$f")
    if [ -z "$src" ]; then
      err LIN-001 "$f inherits workspace lints but no workspace Cargo.toml was found"
      continue
    fi
    own=$(lints "$src")
    sec_clippy=workspace.lints.clippy sec_rust=workspace.lints.rust
    [ "$src" = "$f" ] || where=" in $src"
  fi
  level() { awk -v s="$1" -v k="$2" '$1 == s && $2 == k { print $3; exit }' <<<"$own"; }
  missing=()
  level_ok "$(level "$sec_clippy" unwrap_used)" warn || missing+=('unwrap_used = "warn"')
  level_ok "$(level "$sec_clippy" expect_used)" warn || missing+=('expect_used = "warn"')
  if [ "${#missing[@]}" -gt 0 ]; then
    err LIN-001 "$f: [$sec_clippy]$where is missing $(printf '%s, ' "${missing[@]}" | sed 's/, $//')"
  fi
  # deny, not forbid, is allowed when a comment just above it says why (the
  # one #[unsafe(...)] a platform entry point or a JIT call needs).
  unsafe_level=$(level "$sec_rust" unsafe_code)
  if [ "$unsafe_level" = deny ] &&
    grep -B1 -E '^[[:space:]]*unsafe_code[[:space:]]*=' "$src" | head -n1 | grep -qE '^[[:space:]]*#'; then
    unsafe_level=forbid
  fi
  level_ok "$unsafe_level" forbid ||
    err LIN-002 "$f: [$sec_rust]$where is missing unsafe_code = \"forbid\""
done

[ "$errors" -eq 0 ]
