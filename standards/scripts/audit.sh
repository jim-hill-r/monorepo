#!/usr/bin/env bash
# Check the repo against its project standards. Exits non-zero on any error.
#
#   STD-001  Every top-level project directory has a mise.toml.
#   STD-002  Every mise.toml (except the root) includes exactly one
#            standards/tasks/<type>.toml, and that file exists.
#   STD-003  Every shared task file defines the standard tasks.
#   STD-004  The root mise.toml lists every project in [monorepo].config_roots.
#   NAM-001  Project directory names are snake_case.
#   HUM-001  TODO (human) lines live only in the root HUMAN_ISSUES.md.
set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

standard_tasks=(ci fix build test dev release deploy)
errors=0
err() { echo "error[$1] $2"; errors=$((errors + 1)); }

# Top-level directories that are not projects.
skip_top='^\.'  # hidden directories are not projects

for d in $(git ls-files | cut -d/ -f1 | sort -u); do
  [ -d "$d" ] || continue
  [[ "$d" =~ $skip_top ]] && continue
  [ -f "$d/mise.toml" ] || err STD-001 "$d has no mise.toml"
done

for f in $(git ls-files -- '*/mise.toml'); do
  d=$(dirname "$f")
  includes=$(grep -oE 'standards/tasks/[a-z0-9-]+\.toml' "$f" | sort -u || true)
  n=$(printf '%s' "$includes" | grep -c . || true)
  if [ "$n" -ne 1 ]; then
    err STD-002 "$f must include exactly one standards/tasks/<type>.toml (found $n)"
  elif [ ! -f "$includes" ]; then
    err STD-002 "$f includes $includes, which does not exist"
  fi
  for part in ${d//\// }; do
    [[ "$part" =~ ^[a-z][a-z0-9_]*$ ]] || err NAM-001 "$d: '$part' is not snake_case"
  done
done

# STD-004: the root mise.toml lists every project in [monorepo].config_roots.
listed=$(sed -n '/^config_roots = \[/,/^\]/p' mise.toml | grep -oE '"[^"]+"' | tr -d '"' | sort)
actual=$(git ls-files -- '*/mise.toml' | xargs -r -n1 dirname | sort -u)
if [ "$listed" != "$actual" ]; then
  err STD-004 "[monorepo].config_roots in mise.toml does not match the project list: $(diff <(echo "$listed") <(echo "$actual") | grep '^[<>]' | tr '\n' ' ')"
fi

# HUM-001: tasks for a person live in HUMAN_ISSUES.md, not in an ISSUES.md.
while IFS= read -r hit; do
  err HUM-001 "$hit: move this TODO (human) line to HUMAN_ISSUES.md"
done < <(git grep -n -F 'TODO (human)' -- '*ISSUES.md' ':!HUMAN_ISSUES.md' || true)

for t in standards/tasks/*.toml; do
  for task in "${standard_tasks[@]}"; do
    grep -qE "^\[\"?$task\"?\]" "$t" || err STD-003 "$t does not define task '$task'"
  done
done

# FT-*: free tier guards (see standards/docs/free-tiers.md).
bash standards/scripts/free-tiers.sh check || errors=$((errors + 1))

if [ "$errors" -gt 0 ]; then
  echo "audit: $errors error(s)"
  exit 1
fi
echo "audit: ok"
