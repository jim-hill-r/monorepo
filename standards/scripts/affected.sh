#!/usr/bin/env bash
# Print a JSON array of project directories (dirs with a mise.toml, other than
# the repo root) that changes since BASE touch.
#
#   affected.sh [--all] [BASE]
#
# BASE defaults to $AFFECTED_BASE, then origin/main. A changed file belongs to
# the deepest project directory above it. Changes to a shared task file
# (standards/tasks/<type>.toml) mark every project that includes it. Changes to
# the root mise.toml, standards/scripts/ or the CI workflows mark every project.
set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

projects() {
  git ls-files -- '*/mise.toml' | xargs -r -n1 dirname | sort -u
}

to_json() { jq -R . | jq -cs .; }

if [ "${1:-}" = "--all" ]; then
  projects | to_json
  exit 0
fi

base="${1:-${AFFECTED_BASE:-origin/main}}"
mapfile -t all < <(projects)
mapfile -t changed < <(git diff --name-only "$base"...HEAD; git diff --name-only HEAD)

declare -A hit=()
for f in "${changed[@]}"; do
  [ -n "$f" ] || continue
  case "$f" in
    mise.toml|standards/scripts/*|.github/workflows/*mise*.yml)
      for p in "${all[@]}"; do hit[$p]=1; done
      continue ;;
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

printf '%s\n' "${!hit[@]}" | sed '/^$/d' | sort | to_json
