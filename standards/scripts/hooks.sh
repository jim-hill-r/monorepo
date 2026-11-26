#!/usr/bin/env bash
# Git hook checks, run by lefthook (see lefthook.yml). Runs what PR CI runs:
# `mise run //:audit`, then `mise run ci` in every affected project.
#
#   hooks.sh pre-commit   projects touched by the staged and unstaged changes;
#                         while a merge is being committed, the projects the
#                         branch changes compared with the branch merged in
#   hooks.sh pre-push     projects touched since the PR base (AFFECTED_BASE,
#                         default origin/main, fetched fresh first)
#
# Projects come from `affected.sh --local`: a change to the root mise.toml,
# standards/scripts/ or the CI workflows checks only the projects that test
# those files, not every project as PR CI does.
#
# Every project runs even after a failure, so one run shows every problem.
# A pre-push skips the projects whose `ci` already passed in a pre-commit of
# the exact tree being pushed, so commit-then-push doesn't run CI twice.
#
# When mise can't install a project's tools (cloud sandboxes often block
# mise's version lookups), it retries the install with MISE_OFFLINE=1, which
# still installs Rust through rustup, and failing that runs ci with whatever
# tools are on PATH. A project whose `ci:check-tools` task fails (a system
# library or download its build needs is missing) is skipped with a note, not
# failed: PR CI checks it. See standards/docs/git-hooks.md.
set -uo pipefail

stage=${1:?usage: hooks.sh pre-commit|pre-push}
root=$(git rev-parse --show-toplevel)
cd "$root"

say() { printf 'hooks: %s\n' "$*" >&2; }

if ! command -v mise >/dev/null 2>&1; then
  say "mise is not installed, so the CI checks can't run."
  say "Install it (https://mise.jdx.dev/getting-started.html, or"
  say "'npm install -g @jdxcode/mise' in a cloud sandbox) and try again."
  exit 1
fi

export MISE_YES=1
export MISE_TRUSTED_CONFIG_PATHS="$root${MISE_TRUSTED_CONFIG_PATHS:+:$MISE_TRUSTED_CONFIG_PATHS}"

# A pass is recorded per tree, so a later push of the same tree can skip it.
cache_dir=$(git rev-parse --git-path ci-hooks)
mkdir -p "$cache_dir"

case "$stage" in
  pre-commit)
    # Merging main in: check what the branch changes, not main's changes.
    if git rev-parse -q --verify MERGE_HEAD >/dev/null; then
      say "merge in progress; checking the branch's changes against MERGE_HEAD"
      changed=$(git diff --name-only MERGE_HEAD)
    else
      changed=$(git diff --name-only HEAD)
    fi
    projects=$(bash standards/scripts/affected.sh --local --files <<<"$changed")
    # The committed tree, if nothing unstaged changes what CI sees.
    if git diff --quiet; then tree=$(git write-tree); else tree=""; fi
    ;;
  pre-push)
    base=${AFFECTED_BASE:-origin/main}
    # A stale origin/main makes everything merged since look changed.
    if [ "$base" = origin/main ]; then
      git fetch --quiet origin main || say "couldn't fetch origin/main; using the local copy"
    fi
    # Cloud sandboxes clone shallow, and a shallow clone has no merge base, so
    # affected.sh would see every file as changed.
    if ! git merge-base "$base" HEAD >/dev/null 2>&1 \
      && [ "$(git rev-parse --is-shallow-repository)" = true ]; then
      say "shallow clone; fetching full history to find the merge base"
      git fetch --quiet --unshallow origin || say "fetch failed; checking every project"
    fi
    projects=$(bash standards/scripts/affected.sh --local "$base")
    tree=$(git rev-parse 'HEAD^{tree}')
    ;;
  *)
    say "unknown stage '$stage' (want pre-commit or pre-push)"
    exit 2
    ;;
esac

mapfile -t list < <(jq -r '.[]' <<<"$projects")
say "$stage: ${#list[@]} project(s): ${list[*]:-none}"

# The hook runs with GIT_DIR and GIT_INDEX_FILE (a temporary index during
# `git commit -a`) set for this repo. Tests that run git in their own temp
# repos would use them too, so clear them now that git work here is done.
unset $(git rev-parse --local-env-vars)

failed=()
skipped=()

# Install the project's tools, then run its ci task. When the install fails
# even offline, run ci with the tools already on PATH (for example a system
# cargo), so a sandbox without network access still gets every check it can.
run_ci() {
  if mise install --quiet; then
    mise run ci
  elif say "$p: tool install failed, retrying with MISE_OFFLINE=1" \
    && MISE_OFFLINE=1 mise install --quiet; then
    mise run ci
  else
    say "$p: tools still not installed; running ci with the tools on PATH"
    MISE_OFFLINE=1 MISE_TASK_RUN_AUTO_INSTALL=false mise run ci
  fi
}

# Run the project's ci:check-tools task, if it has one, without installing
# tools. It fails, naming what is missing, when the sandbox lacks a system
# tool or download that ci needs.
check_tools() {
  mise tasks info ci:check-tools >/dev/null 2>&1 || return 0
  MISE_OFFLINE=1 MISE_TASK_RUN_AUTO_INSTALL=false mise run --quiet ci:check-tools
}

say "running //:audit"
mise run --quiet //:audit || failed+=("//:audit")

for p in "${list[@]}"; do
  if [ "$stage" = pre-push ] && grep -qxF "$p" "$cache_dir/$tree" 2>/dev/null; then
    say "$p: ci already passed for this tree in pre-commit, skipping"
    continue
  fi
  if ! why=$(cd "$p" && check_tools 2>&1); then
    say "$p: not checked here, PR CI checks it: ${why:-ci:check-tools failed}"
    skipped+=("$p")
    continue
  fi
  say "$p: mise run ci"
  if ! (cd "$p" && run_ci); then
    failed+=("$p")
    continue
  fi
  [ -z "$tree" ] || echo "$p" >>"$cache_dir/$tree"
done

if [ ${#failed[@]} -gt 0 ]; then
  say "FAILED: ${failed[*]}"
  say "Fix them (mise run fix helps with format and lint) and try again."
  exit 1
fi
if [ ${#skipped[@]} -gt 0 ]; then
  say "not checked here (tools missing), PR CI checks them: ${skipped[*]}"
  say "Say so in the PR."
fi
say "$stage: all checks passed"
