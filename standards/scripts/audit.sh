#!/usr/bin/env bash
# Check the repo against its project standards. Exits non-zero on any error.
#
#   STD-001  Every top-level project directory has a mise.toml.
#   STD-002  Every mise.toml (except the root) includes exactly one
#            standards/tasks/<type>.toml, and that file exists.
#   STD-003  Every shared task file defines the standard tasks.
#   STD-004  The root mise.toml lists every project in [monorepo].config_roots.
#   STD-005  No tracked file holds an unresolved merge conflict marker
#            (`chores conflict-markers`, Rust).
#   NAM-001  Project directory names are snake_case.
#   NAM-002, NAM-004, DOC-*, LIN-*  Package names, README.md and
#            CONTRIBUTING.md, Cargo lints (project-check.sh); DOC-* and
#            LIN-* are warnings only for now.
#   DOC-005 to DOC-008  User guides against the baseline in
#            standards/user-docs.toml (`chores user-docs`, Rust).
#   ISS-*    Issue files (issues.sh --check): one file per issue, valid
#            front matter, human tasks only in issues/human/.
#   EVL-*    Requirements and evals (`chores evals-check`); warnings only for now.
#   COST-*   Each project's docs/cost.toml (cost-check.sh); warnings only for now.
#   ROY-001, RTOML-*  Each project's royalty.toml (royalty-check.sh).
#   THR-*    Each project's docs/threat.toml (`chores threat-check`); warnings only for now.
#   HLT-001  The Repo health tables in README.md and
#            standards/docs/repo-health.md match standards/repo-health.toml
#            (`chores repo-health`), the one source pane reads too.
#   QUAL-404 No new tracked .sh files: the ones left to port are listed in
#            standards/shell-scripts.toml (`chores shell-scripts`).
#   RES-*    Resources (resources.sh check, standards/docs/resources.md):
#            dropped claims retired (with a base), no references to
#            retired names, artifact retention, valid resources.toml.
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

# NAM-002, NAM-004 (errors), DOC-*, LIN-* (warnings): per-project checks.
bash standards/scripts/project-check.sh || errors=$((errors + 1))

chores=(cargo run --quiet --manifest-path chores/Cargo.toml --)
if command -v cargo >/dev/null 2>&1; then
  # DOC-005 to DOC-008: every project's user guide (standards/docs/documentation.md#user-docs).
  "${chores[@]}" user-docs || errors=$((errors + 1))
  # HLT-001: the README badges and pane read one source.
  "${chores[@]}" repo-health || errors=$((errors + 1))
  # STD-005: no unresolved merge conflict markers in tracked files.
  "${chores[@]}" conflict-markers || errors=$((errors + 1))
  # QUAL-404: no new shell scripts (standards/shell-scripts.toml).
  "${chores[@]}" shell-scripts || errors=$((errors + 1))
  # EVL-*: requirements and evals. Warnings only; never fails the audit.
  "${chores[@]}" evals-check || true
else
  echo "warning: user docs, conflict marker, shell script, evals and threat checks skipped: cargo is not installed"
fi

# COST-*: per-project cost analyses against the budgets. Warnings only.
bash standards/scripts/cost-check.sh

# ROY-001, RTOML-*: every project's royalty.toml.
bash standards/scripts/royalty-check.sh || errors=$((errors + 1))

# THR-*: per-project threat models against the security rules. Warnings
# only; COST_PROJECTS (or THREAT_PROJECTS) picks the affected projects.
if command -v cargo >/dev/null 2>&1; then
  "${chores[@]}" threat-check || true
fi

# SEC-002, SEC-006: repo-wide security guards (see standards/docs/security.md).
bash standards/scripts/security-check.sh || errors=$((errors + 1))

# DS-*: no raw colours in projects that adopted the design system.
bash standards/scripts/design-check.sh || errors=$((errors + 1))

# RES-*: resource claims and retirements (see standards/docs/resources.md).
# RES-001 runs when RESOURCES_BASE or AFFECTED_BASE names the PR base.
bash standards/scripts/resources.sh check || errors=$((errors + 1))

# FT-*: free tier guards (see standards/docs/free-tiers.md).
bash standards/scripts/free-tiers.sh check || errors=$((errors + 1))

if [ "$errors" -gt 0 ]; then
  echo "audit: $errors error(s)"
  exit 1
fi
echo "audit: ok"
