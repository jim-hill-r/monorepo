#!/usr/bin/env bash
# Decide whether the checked-out commit may deploy PROJECT. Only the latest
# version of main that passed CI is deployed (Jim, 2026-11-26), so a run whose
# commit is no longer the newest one for PROJECT must not overwrite it.
#
#   deploy-check.sh PROJECT
#
# Run after `mise run ci` has passed for PROJECT. Exit status:
#   0  deploy: HEAD is main's tip, or no newer commit on main affects PROJECT
#   3  skip: a newer commit on main affects PROJECT (its own push run deploys
#      it), or HEAD is not on main; the reason is printed. A project whose
#      mise.toml has a `# deploy-on-every-push` line is affected by every
#      commit, so only main's tip deploys it
#   other  error
#
# "Affects" is decided by affected.sh, run at main's tip against HEAD, so
# path dependencies and `# affected-by:` lines count too.
set -euo pipefail

project=${1:?usage: deploy-check.sh PROJECT}
cd "$(git rev-parse --show-toplevel)"

head=$(git rev-parse HEAD)
git fetch --quiet --no-tags --depth="${DEPLOY_CHECK_DEPTH:-200}" origin \
  +refs/heads/main:refs/remotes/origin/main
tip=$(git rev-parse refs/remotes/origin/main)

if [ "$head" = "$tip" ]; then
  echo "deploy: $head is the tip of main"
  exit 0
fi

if ! git merge-base --is-ancestor "$head" "$tip" 2>/dev/null; then
  echo "skip: $head is not in the last ${DEPLOY_CHECK_DEPTH:-200} commits of main"
  exit 3
fi

if grep -qE '^# *deploy-on-every-push' "$project/mise.toml" 2>/dev/null; then
  echo "skip: main moved to $tip, and $project deploys on every push; that commit's run deploys it"
  exit 3
fi

worktree=$(mktemp -d)
trap 'git worktree remove --force "$worktree" >/dev/null 2>&1 || rm -rf "$worktree"' EXIT
git worktree add --quiet --detach "$worktree" "$tip"
affected=$(cd "$worktree" && bash standards/scripts/affected.sh "$head")

if jq -e --arg p "$project" 'index($p)' <<<"$affected" >/dev/null; then
  echo "skip: main moved to $tip, which changes $project; that commit's run deploys it"
  exit 3
fi
echo "deploy: main moved to $tip, but nothing newer changes $project"
