#!/usr/bin/env bash
# Print a JSON array of project directories (dirs with a mise.toml, other than
# the repo root) that changes since BASE touch.
#
#   affected.sh [--all] [--changed-only] [BASE | --files]
#
# BASE defaults to $AFFECTED_BASE, then origin/main. An empty BASE, or one of
# all zeros (a new branch push), lists every project. With --files the changed
# paths are read from stdin, one per line, instead of from git.
#
# A project is affected when:
#   - a changed file is inside it and in no deeper project;
#   - a shared task file it includes (standards/tasks/<type>.toml) changed;
#   - a Cargo path dependency of it changed. Path dependencies are read from
#     every Cargo.toml in the project and in the directories above it, so a
#     workspace member sees the workspace's path dependencies;
#   - a directory named on an `# affected-by: <dir>` line in its mise.toml
#     changed, or is itself an affected project (for example a cloudflare
#     project names the web app it deploys).
# Changes to the root mise.toml, standards/scripts/ or the mise CI workflows
# affect every project, except with --changed-only (or AFFECTED_CHANGED_ONLY=1),
# which PR CI and the git hooks use. Then such a change affects only:
#   - the projects that test those files (`standards` and `monorepo`, through
#     their own paths and affected-by lines);
#   - for a script in standards/scripts/, the projects whose mise.toml or
#     included standards/tasks file names it.
# Pushes to main and the nightly runs still check every project.
set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

all_flag="" changed_only="${AFFECTED_CHANGED_ONLY:-}" files_flag="" base_arg=""
while [ $# -gt 0 ]; do
  case "$1" in
    --all) all_flag=1 ;;
    --changed-only) changed_only=1 ;;
    --files) files_flag=1 ;;
    *) base_arg=$1 ;;
  esac
  shift
done

projects() {
  git ls-files -- '*/mise.toml' | xargs -r -n1 dirname | sort -u
}

to_json() { jq -R . | jq -cs .; }

if [ -n "$all_flag" ]; then
  projects | to_json
  exit 0
fi

if [ -n "$files_flag" ]; then
  mapfile -t changed
else
  base="${base_arg:-${AFFECTED_BASE:-origin/main}}"
  if [ -z "$base" ] || [[ "$base" =~ ^0+$ ]]; then
    projects | to_json
    exit 0
  fi
  mapfile -t changed < <(git diff --name-only "$base"...HEAD; git diff --name-only HEAD)
fi

mapfile -t all < <(projects)

# Normalise a path relative to the repo root.
norm() { realpath -m --relative-to=. "$1"; }

# Directories outside project $1 whose changes affect it.
watched_dirs() {
  local p=$1 f d dep
  {
    git ls-files -- "$p/Cargo.toml" "$p/**/Cargo.toml"
    d=$(dirname "$p")
    while [ "$d" != "." ]; do
      [ -f "$d/Cargo.toml" ] && echo "$d/Cargo.toml"
      d=$(dirname "$d")
    done
  } | while read -r f; do
    { grep -E '^[A-Za-z0-9_-]+ *= *\{[^}]*path *= *"' "$f" || true; } \
      | sed -E 's/.*path *= *"([^"]+)".*/\1/' \
      | while read -r dep; do norm "$(dirname "$f")/$dep"; done
  done
  { grep -E '^# *affected-by:' "$p/mise.toml" || true; } \
    | sed -E 's/^# *affected-by: *//' \
    | while read -r dep; do norm "$p/$dep"; done
}

# Whether project $1's mise.toml, or a standards/tasks file it includes,
# names the script $2.
uses_script() {
  local t
  grep -qF "$2" "$1/mise.toml" && return 0
  for t in $(grep -oE 'standards/tasks/[A-Za-z0-9_-]+\.toml' "$1/mise.toml" | sort -u); do
    [ -f "$t" ] && grep -qF "$2" "$t" && return 0
  done
  return 1
}

declare -A hit=()
declare -A watch=()
for p in "${all[@]}"; do
  watch[$p]=$(watched_dirs "$p" | grep -v "^$p\(/\|$\)" | sort -u || true)
done

for f in "${changed[@]}"; do
  [ -n "$f" ] || continue
  case "$f" in
    mise.toml|standards/scripts/*|.github/workflows/pull-request-ci.yml|.github/workflows/push-main.yml|.github/workflows/nightly-ci.yml|.github/workflows/nightly-cd.yml|.github/workflows/mise-projects.yml)
      if [ -z "$changed_only" ] || [ "$changed_only" = 0 ]; then
        for p in "${all[@]}"; do hit[$p]=1; done
        continue
      fi
      case "$f" in
        standards/scripts/*)
          for p in "${all[@]}"; do
            uses_script "$p" "$f" && hit[$p]=1
          done ;;
      esac ;;
    standards/tasks/*.toml)
      t=$(basename "$f")
      for p in "${all[@]}"; do
        grep -q "standards/tasks/$t" "$p/mise.toml" && hit[$p]=1
      done
      continue ;;
  esac
  best=""
  for p in "${all[@]}"; do
    case "$f" in
      "$p"/*) [ ${#p} -gt ${#best} ] && best=$p ;;
    esac
  done
  [ -n "$best" ] && hit[$best]=1
done

# Follow path dependencies and affected-by lines until nothing new is hit.
# Changed files plus every affected project's directory count as changes.
touched() {
  printf '%s\n' "${changed[@]}"
  for p in "${!hit[@]}"; do echo "$p/"; done
}
while :; do
  mapfile -t now < <(touched)
  added=0
  for p in "${all[@]}"; do
    [ -n "${hit[$p]:-}" ] && continue
    [ -n "${watch[$p]}" ] || continue
    while read -r w; do
      for f in "${now[@]}"; do
        case "$f" in
          "$w"/*|"$w") hit[$p]=1; added=1; break 2 ;;
        esac
      done
    done <<< "${watch[$p]}"
  done
  [ $added -eq 1 ] || break
done

printf '%s\n' "${!hit[@]}" | sed '/^$/d' | sort | to_json
