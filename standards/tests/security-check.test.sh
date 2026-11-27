#!/usr/bin/env bash
# Tests standards/scripts/security-check.sh (SEC-002, SEC-006) in throwaway
# repos: a good setup passes, and each kind of gap fails with its code.
#   bash standards/tests/security-check.test.sh
set -uo pipefail

root=$(cd "$(dirname "$0")/../.." && pwd)
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
failures=0

# new_repo NAME: a repo with the real .gitignore, one Cargo project and one
# npm project, and a dependabot.yml covering both.
new_repo() {
  local repo=$work/$1
  mkdir -p "$repo/standards/scripts" "$repo/.github" "$repo/app/web"
  cp "$root/standards/scripts/security-check.sh" "$repo/standards/scripts/"
  cp "$root/.gitignore" "$repo/"
  echo '# lock' >"$repo/app/Cargo.lock"
  echo '{}' >"$repo/app/web/package-lock.json"
  cat >"$repo/.github/dependabot.yml" <<'YML'
version: 2
updates:
  - package-ecosystem: "cargo"
    directory: "/app"
  - package-ecosystem: "npm"
    directory: "/app/web"
  - package-ecosystem: "github-actions"
    directory: "/"
YML
  git -C "$repo" init -q
  echo "$repo"
}

add_all() { git -C "$1" add -A && git -C "$1" add -f "${@:2}" 2>/dev/null; true; }

# expect NAME REPO WANT_STATUS [WANT_TEXT]
expect() {
  local name=$1 repo=$2 want=$3 text=${4:-}
  out=$(cd "$repo" && bash standards/scripts/security-check.sh 2>&1)
  local got=$?
  if [ "$got" -ne "$want" ] || { [ -n "$text" ] && [[ "$out" != *"$text"* ]]; }; then
    echo "FAIL $name: exit $got (want $want), want text '$text' in:"
    echo "$out" | sed 's/^/    /'
    failures=$((failures + 1))
  fi
}

r=$(new_repo ok); add_all "$r"
echo 'X=1' >"$r/app/.dev.vars.example"; add_all "$r"
expect "good setup" "$r" 0

r=$(new_repo ignore); add_all "$r"
grep -v -e '\.dev\.vars' "$r/.gitignore" >"$r/.gi" && mv "$r/.gi" "$r/.gitignore"
expect "missing .dev.vars pattern" "$r" 1 "error[SEC-002] .gitignore does not ignore .dev.vars"

r=$(new_repo tracked); echo 'k' >"$r/app/release.keystore"; add_all "$r" app/release.keystore
expect "tracked keystore" "$r" 1 "error[SEC-002] app/release.keystore is tracked"

r=$(new_repo stale); add_all "$r"
printf '  - package-ecosystem: "cargo"\n    directory: "/gone"\n' >>"$r/.github/dependabot.yml"
expect "stale directory" "$r" 1 "error[SEC-006] .github/dependabot.yml: cargo entry for /gone, which does not exist"

r=$(new_repo uncovered); mkdir -p "$r/other"; echo '# lock' >"$r/other/Cargo.lock"; add_all "$r"
expect "lockfile without entry" "$r" 1 "error[SEC-006] other/Cargo.lock has no cargo entry"

r=$(new_repo member); mkdir -p "$r/app/member"; add_all "$r"
printf '  - package-ecosystem: "cargo"\n    directory: "/app/member"\n' >>"$r/.github/dependabot.yml"
expect "workspace member entry" "$r" 1 "which has no tracked Cargo.lock"

if [ "$failures" -gt 0 ]; then
  echo "security-check.test.sh: $failures failure(s)"
  exit 1
fi
echo "security-check.test.sh: all cases passed"
