#!/usr/bin/env bash
# Check the repo against its project standards. Exits non-zero on any error.
#
#   STD-001  Every top-level project directory has a mise.toml.
#   STD-002  Every mise.toml (except the root) includes exactly one
#            standards/tasks/<type>.toml, and that file exists.
#   STD-003  Every shared task file defines the standard tasks.
#   STD-004  The root mise.toml lists every project in [monorepo].config_roots.
#   NAM-001  Project directory names are snake_case.
#   ISS-*    Issue files (issues.sh --check): one file per issue, valid
#            front matter, human tasks only in issues/human/.
#   EVL-*    Requirements and evals (evals-check.sh); warnings only for now.
#   COST-*   Each project's docs/cost.toml (cost-check.sh); warnings only for now.
#   SEC-*    Secret files ignored and Dependabot covering every lockfile
#            (security-check.sh, standards/docs/security.md).
#   DS-*     Projects that adopted the design system use its tokens, not raw
#            colours (design-check.sh).
set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

standard_tasks=(ci fix build test dev release deploy evals)
errors=0
err() { echo "error[$1] $2"; errors=$((errors + 1)); }

# Top-level directories that are not projects.
skip_top='^(\.[^/]*|docs|issues)$'  # hidden directories are not projects; docs/ holds only review decks (docs/reviews), issues/ only repo-wide issue files

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

# ISS-*: issue files (see standards/docs/issue-management.md).
bash standards/scripts/issues.sh --check || errors=$((errors + 1))

for t in standards/tasks/*.toml; do
  for task in "${standard_tasks[@]}"; do
    grep -qE "^\[\"?$task\"?\]" "$t" || err STD-003 "$t does not define task '$task'"
  done
done

# EVL-*: requirements and evals. Warnings only; never fails the audit.
bash standards/scripts/evals-check.sh

# COST-*: per-project cost analyses against the budgets. Warnings only.
bash standards/scripts/cost-check.sh

# SEC-002, SEC-006: repo-wide security guards (see standards/docs/security.md).
bash standards/scripts/security-check.sh || errors=$((errors + 1))

# DS-*: no raw colours in projects that adopted the design system.
bash standards/scripts/design-check.sh || errors=$((errors + 1))

# FT-*: free tier guards (see standards/docs/free-tiers.md).
bash standards/scripts/free-tiers.sh check || errors=$((errors + 1))

if [ "$errors" -gt 0 ]; then
  echo "audit: $errors error(s)"
  exit 1
fi
echo "audit: ok"
