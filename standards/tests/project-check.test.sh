#!/usr/bin/env bash
# Tests standards/scripts/project-check.sh (NAM-002, NAM-004, DOC-001 to
# DOC-004, LIN-001, LIN-002) in throwaway repos.
#   bash standards/tests/project-check.test.sh
set -uo pipefail

root=$(cd "$(dirname "$0")/../.." && pwd)
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
failures=0

# docs DIR NAME: a README.md with a NAME heading and a CONTRIBUTING.md with
# Getting Started.
docs() {
  printf '# %s\n\nWhat it is.\n' "$2" >"$1/README.md"
  printf '# Contributing\n\n## Getting Started\n\nmise run ci\n' >"$1/CONTRIBUTING.md"
}

# new_repo NAME: a repo where every check passes: a Rust project (app), a
# workspace whose member inherits its lints (tool, tool/cli), and an npm
# sub-project named parent_dir (app/web).
new_repo() {
  local repo=$work/$1
  mkdir -p "$repo/standards/scripts" "$repo/app/web" "$repo/tool/cli"
  cp "$root/standards/scripts/project-check.sh" "$repo/standards/scripts/"
  echo '[tools]' >"$repo/mise.toml"
  for d in app app/web tool tool/cli; do echo '[tools]' >"$repo/$d/mise.toml"; done
  cat >"$repo/app/Cargo.toml" <<'TOML'
[package]
name = "app"
version = "0.1.0"

[lints.clippy]
unwrap_used = "warn"
expect_used = { level = "deny", priority = 1 }

[lints.rust]
unsafe_code = "forbid"
TOML
  printf '{\n  "name": "app_web",\n  "private": true\n}\n' >"$repo/app/web/package.json"
  cat >"$repo/tool/Cargo.toml" <<'TOML'
[workspace]
members = ["cli"]

[workspace.lints.clippy]
unwrap_used = "warn"
expect_used = "warn"

[workspace.lints.rust]
unsafe_code = "forbid"
TOML
  cat >"$repo/tool/cli/Cargo.toml" <<'TOML'
[package]
name = "tool_cli"

[lints]
workspace = true
TOML
  docs "$repo/app" app
  docs "$repo/app/web" app_web
  docs "$repo/tool" tool
  docs "$repo/tool/cli" cli
  git -C "$repo" init -q
  echo "$repo"
}

add_all() { git -C "$1" add -A; }

# expect NAME REPO WANT_STATUS [WANT_TEXT] [UNWANTED_TEXT]
expect() {
  local name=$1 repo=$2 want=$3 text=${4:-} unwanted=${5:-}
  out=$(cd "$repo" && bash standards/scripts/project-check.sh 2>&1)
  local got=$?
  if [ "$got" -ne "$want" ] || { [ -n "$text" ] && [[ "$out" != *"$text"* ]]; } ||
    { [ -n "$unwanted" ] && [[ "$out" == *"$unwanted"* ]]; }; then
    echo "FAIL $name: exit $got (want $want), want '$text', not '$unwanted', in:"
    echo "$out" | sed 's/^/    /'
    failures=$((failures + 1))
  fi
}

r=$(new_repo ok); add_all "$r"
expect "every check passes" "$r" 0 "" "["

r=$(new_repo nam2); sed -i 's/name = "app"/name = "my_app"/' "$r/app/Cargo.toml"; add_all "$r"
expect "Cargo package name differs from its directory" "$r" 1 "error[NAM-002] app: package name 'my_app'"

r=$(new_repo nam2npm); sed -i 's/"app_web"/"web-ui"/' "$r/app/web/package.json"; add_all "$r"
expect "npm package name differs from its directory" "$r" 1 "error[NAM-002] app/web: package name 'web-ui' must be 'web' or 'app_web'"

r=$(new_repo nam4); mkdir -p "$r/proof"; echo '[tools]' >"$r/proof/mise.toml"; docs "$r/proof" proof; add_all "$r"
expect "proof of concept without poc_" "$r" 1 "error[NAM-004] proof: proof of concept project 'proof'"

r=$(new_repo nam4ok); mkdir -p "$r/poc_idea"; echo '[tools]' >"$r/poc_idea/mise.toml"; docs "$r/poc_idea" poc_idea; add_all "$r"
expect "proof of concept named poc_" "$r" 0 "" "NAM-004"

r=$(new_repo pocket); mkdir -p "$r/pocket"; echo '[tools]' >"$r/pocket/mise.toml"; docs "$r/pocket" pocket; add_all "$r"
expect "a name that only starts with poc is not a PoC" "$r" 0 "" "NAM-004"

r=$(new_repo doc1); rm "$r/app/README.md"; add_all "$r"
expect "no README.md is an error" "$r" 1 "error[DOC-001] app has no README.md"

r=$(new_repo doc2); echo 'No heading here.' >"$r/app/README.md"; add_all "$r"
expect "README.md without the project heading" "$r" 1 "error[DOC-002] app/README.md has no '# app' heading"

r=$(new_repo doc2dir); add_all "$r"
expect "a heading with the directory name is enough" "$r" 0 "" "DOC-002"

r=$(new_repo doc2path); printf '# app/web\n\nWhat it is.\n' >"$r/app/web/README.md"; add_all "$r"
expect "a heading with the project's path is enough" "$r" 0 "" "DOC-002"

r=$(new_repo doc2spaces); mkdir -p "$r/psy_ents"; echo '[tools]' >"$r/psy_ents/mise.toml"; docs "$r/psy_ents" "Psy Ents"; add_all "$r"
expect "a heading with spaces for underscores is enough" "$r" 0 "" "DOC-002"

r=$(new_repo doc2the); mkdir -p "$r/psy_ents"; echo '[tools]' >"$r/psy_ents/mise.toml"; docs "$r/psy_ents" "The Psy Ents"; add_all "$r"
expect "a spaced heading after The is enough" "$r" 0 "" "DOC-002"

r=$(new_repo held); mkdir -p "$r/standards/docs/issues" "$r/old/sub"; echo '[tools]' >"$r/old/mise.toml"; echo '[tools]' >"$r/old/sub/mise.toml"
printf '[package]\nname = "old"\n' >"$r/old/Cargo.toml"
printf '# Priority Projects\n- app\n\n# On Hold Projects\n- old\n\n# Epic notes\n- tool\n' >"$r/standards/docs/issues/README.md"; add_all "$r"
expect "on hold projects are exempt from DOC and LIN" "$r" 0 "" "old"

r=$(new_repo notheld); mkdir -p "$r/standards/docs/issues"; rm "$r/tool/CONTRIBUTING.md"
printf '# On Hold Projects\n- old\n\n# Epic notes\n- tool\n' >"$r/standards/docs/issues/README.md"; add_all "$r"
expect "only the On Hold list exempts a project" "$r" 1 "error[DOC-003] tool has no CONTRIBUTING.md"

r=$(new_repo doc3); rm "$r/tool/CONTRIBUTING.md"; add_all "$r"
expect "no CONTRIBUTING.md is an error" "$r" 1 "error[DOC-003] tool has no CONTRIBUTING.md"

r=$(new_repo doc4); echo '# Contributing' >"$r/app/CONTRIBUTING.md"; add_all "$r"
expect "CONTRIBUTING.md without Getting Started" "$r" 1 "error[DOC-004] app/CONTRIBUTING.md has no 'Getting Started' heading"

r=$(new_repo lin1); sed -i '/^unwrap_used/d' "$r/app/Cargo.toml"; add_all "$r"
expect "missing unwrap_used" "$r" 1 "error[LIN-001] app/Cargo.toml: [lints.clippy] is missing unwrap_used = \"warn\""

r=$(new_repo lin1allow); sed -i 's/^unwrap_used = "warn"/unwrap_used = "allow"/' "$r/app/Cargo.toml"; add_all "$r"
expect "unwrap_used allowed is too weak" "$r" 1 "error[LIN-001] app/Cargo.toml"

r=$(new_repo lin1ws); sed -i '/^expect_used/d' "$r/tool/Cargo.toml"; add_all "$r"
expect "workspace missing expect_used" "$r" 1 "error[LIN-001] tool/cli/Cargo.toml: [workspace.lints.clippy] in tool/Cargo.toml is missing expect_used"

r=$(new_repo lin1nows); mkdir -p "$r/lone"; echo '[tools]' >"$r/lone/mise.toml"; docs "$r/lone" lone
printf '[package]\nname = "lone"\n\n[lints]\nworkspace = true\n' >"$r/lone/Cargo.toml"; add_all "$r"
expect "inherits lints with no workspace" "$r" 1 "error[LIN-001] lone/Cargo.toml inherits workspace lints but no workspace Cargo.toml was found"

r=$(new_repo lin1root); mkdir -p "$r/both"; echo '[tools]' >"$r/both/mise.toml"; docs "$r/both" both
cat >"$r/both/Cargo.toml" <<'TOML'
[package]
name = "both"

[workspace]

[workspace.lints.clippy]
unwrap_used = "warn"
expect_used = "warn"

[workspace.lints.rust]
unsafe_code = "forbid"

[lints]
workspace = true
TOML
add_all "$r"
expect "a package that is its own workspace root" "$r" 0 "" "LIN-"

r=$(new_repo lin2); sed -i 's/unsafe_code = "forbid"/unsafe_code = "deny"/' "$r/app/Cargo.toml"; add_all "$r"
expect "unsafe_code deny without a reason" "$r" 1 "error[LIN-002] app/Cargo.toml: [lints.rust] is missing unsafe_code = \"forbid\""

r=$(new_repo lin2why); sed -i 's/unsafe_code = "forbid"/# deny, not forbid: one #[unsafe(no_mangle)] entry point.\nunsafe_code = "deny"/' "$r/app/Cargo.toml"; add_all "$r"
expect "unsafe_code deny with a reason" "$r" 0 "" "LIN-002"

r=$(new_repo untracked); mkdir -p "$r/scratch"; echo '[tools]' >"$r/scratch/mise.toml"
git -C "$r" add -A ':!scratch'
expect "untracked projects are not checked" "$r" 0 "" "scratch"

if [ "$failures" -gt 0 ]; then
  echo "project-check.test.sh: $failures failure(s)"
  exit 1
fi
echo "project-check.test.sh: ok"
