#!/usr/bin/env bash
# Tests standards/scripts/hooks.sh (with affected.sh) in a throwaway repo:
# which projects a commit or a merge checks (and that the audit's cost check
# gets the same list in COST_PROJECTS), the skip when ci:check-tools
# fails, that git's hook variables don't reach a project's ci, and that a
# staged secret fails the commit (when gitleaks can be installed). Each
# project's ci only logs its name, so this needs only git, jq and mise.
#   bash standards/tests/hooks.test.sh
set -uo pipefail

root=$(cd "$(dirname "$0")/../.." && pwd)
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
repo=$work/repo
log=$work/ci.log
failures=0

git_() {
  git -C "$repo" -c user.name=t -c user.email=t@example.invalid \
    -c init.defaultBranch=main "$@" >/dev/null 2>&1
}

mkdir -p "$repo/standards/scripts"
cp "$root/standards/scripts/hooks.sh" "$root/standards/scripts/affected.sh" \
  "$root/standards/scripts/secrets-check.sh" "$root/standards/scripts/deps-check.sh" \
  "$repo/standards/scripts/"
cp "$root/.gitleaks.toml" "$root/.gitleaksignore" "$repo/"
cat >"$repo/mise.toml" <<'TOML'
monorepo_root = true

[settings]
experimental = true

[monorepo]
config_roots = ["a", "b", "nolib"]

[tasks.audit]
run = 'echo "${COST_PROJECTS-unset}" >>"$AUDIT_LOG"'
TOML
# Each project's ci logs its name, and fails if a git hook variable leaked.
for p in a b nolib; do
  mkdir -p "$repo/$p"
  cat >"$repo/$p/mise.toml" <<TOML
[tasks.ci]
run = '''
if env | grep -qE '^GIT_(DIR|INDEX_FILE)='; then echo "$p: GIT_ vars leaked" >&2; exit 1; fi
echo $p >>"$log"
'''
TOML
  echo 0 >"$repo/$p/file"
done
# b's ci names a shared script, so changing that script checks b.
echo 'true' >"$repo/standards/scripts/used-by-b.sh"
echo '# uses standards/scripts/used-by-b.sh' >>"$repo/b/mise.toml"
cat >>"$repo/nolib/mise.toml" <<'TOML'

[tasks."ci:check-tools"]
run = "echo 'libfoo missing'; exit 1"
TOML
git_ init
git_ add -A
git_ commit -m init

export MISE_TRUSTED_CONFIG_PATHS=$repo MISE_YES=1
# The audit logs the COST_PROJECTS it gets, so the cost check's per-project
# rules can be checked to run for the affected projects only.
export AUDIT_LOG=$work/audit.log

# audited NAME WANT: the COST_PROJECTS the last check's audit saw.
audited() {
  local got
  got=$(tail -n 1 "$AUDIT_LOG")
  if [ "$got" != "$2" ]; then
    echo "FAIL $1: audit got COST_PROJECTS='$got', want '$2'"
    failures=$((failures + 1))
  fi
}

# check NAME WANT_STATUS WANT_PROJECTS [ENV...]: run the pre-commit check and
# compare its exit status and the projects whose ci ran (space separated).
check() {
  local name=$1 want=$2 want_ran=$3
  shift 3
  : >"$log"
  out=$(cd "$repo" && env "$@" bash standards/scripts/hooks.sh pre-commit 2>&1)
  local got=$? ran
  ran=$(sort "$log" | tr '\n' ' ' | sed 's/ $//')
  if [ "$got" -ne "$want" ] || [ "$ran" != "$want_ran" ]; then
    echo "FAIL $name: exit $got (want $want), ran '$ran' (want '$want_ran')"
    echo "$out" | sed 's/^/    /'
    failures=$((failures + 1))
  fi
}

# A change in one project checks only that project.
echo 1 >"$repo/a/file"
check "one project" 0 "a"
audited "one project" "a"
git_ commit -am a

# The root mise.toml and the hook scripts no longer mean every project.
echo "# x" >>"$repo/mise.toml"
echo "# x" >>"$repo/standards/scripts/affected.sh"
check "root files" 0 ""
audited "root files" ""
git_ checkout -- .

# A shared script checks only the projects that name it.
echo "# x" >>"$repo/standards/scripts/used-by-b.sh"
check "used script" 0 "b"
git_ checkout -- .

# A project whose ci:check-tools fails is skipped with a note, not failed.
echo 1 >"$repo/nolib/file"
check "missing tool" 0 ""
case "$out" in
  *"nolib: not checked here, PR CI checks it: libfoo missing"*) ;;
  *) echo "FAIL missing tool: no skip note in:"; echo "$out" | sed 's/^/    /'
     failures=$((failures + 1)) ;;
esac
git_ checkout -- .

# Git's hook variables are cleared before ci runs.
echo 2 >"$repo/a/file"
check "git env" 0 "a" GIT_DIR="$repo/.git" GIT_INDEX_FILE="$repo/.git/index"
git_ checkout -- .

# A staged secret fails the commit, and every project's ci still runs.
# Skipped where gitleaks can't be installed (secrets-check.sh then warns).
if (cd "$repo" && bash standards/scripts/secrets-check.sh history 2>&1) | grep -q 'warning\[SEC-001\]'; then
  echo "hooks.test.sh: gitleaks not available, skipping the staged secret case"
else
  echo 4 >"$repo/a/file"
  # A made-up PostHog personal key, built here so this file has no secret.
  echo "POSTHOG_KEY=phx_$(printf 'Ab3dEf6hIj9kLm2nOp5qRs8tUv1wXy4zAb7cDe0fGh3iJk6')" >"$repo/a/key.txt"
  git_ add a/key.txt
  check "staged secret" 1 "a"
  case "$out" in
    *"error[SEC-001]"*) ;;
    *) echo "FAIL staged secret: no SEC-001 error in:"; echo "$out" | sed 's/^/    /'
       failures=$((failures + 1)) ;;
  esac
  git_ rm --cached -q a/key.txt
  rm -f "$repo/a/key.txt"
  git_ checkout -- .
fi

# Committing a merge of main checks the branch's own changes, not main's.
git_ checkout -b feature
echo 3 >"$repo/a/file"
git_ commit -am feature
git_ checkout main
echo 1 >"$repo/b/file"
echo "# y" >>"$repo/mise.toml"
git_ commit -am main
git_ checkout feature
git_ merge --no-commit --no-ff main
check "merge" 0 "a"
git_ merge --abort

if [ "$failures" -gt 0 ]; then
  echo "hooks.test.sh: $failures failure(s)"
  exit 1
fi
echo "hooks.test.sh: all cases passed"
