#!/usr/bin/env bash
# List or check the issue files (one Markdown file per issue, see
# standards/docs/issue-management.md). The list is generated on demand and
# never committed, so it can't cause merge conflicts.
#
#   issues.sh            Agent issues: priority first, then backlog, by
#                        project, epic and order.
#   issues.sh --human    Human issues (every project's docs/issues/human/),
#                        oldest first.
#   issues.sh --check    Validate every issue file; used by the audit.
#
#   ISS-001  ISSUES.md and HUMAN_ISSUES.md are retired, and so are the root
#            issues/ folder and <project>/issues/ folders: issue files live
#            only in <project>/docs/issues/.
#   ISS-002  Every issue file starts with front matter and has the required
#            fields with allowed values.
#   ISS-003  Human tasks (tag `human`) live only in a docs/issues/human/.
#   ISS-004  Every human task has `## Requirement`, `## Why it can't be
#            automated` and `## Runbook` sections, and a `runbook` front
#            matter field naming a file in standards/docs/runbooks/ that
#            exists.
set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

mode=${1:-list}

# Issue files: <project>/**/docs/issues/*.md and .../docs/issues/human/*.md,
# never README.md. Tracked and new (untracked, not ignored) files both
# count, so the list is right before a commit.
issue_files() {
  git ls-files --cached --others --exclude-standard -- '*/docs/issues/*.md' |
    grep -E '^[^/]+/(.*/)?docs/issues/(human/)?[^/]+\.md$' |
    grep -v '/README\.md$' | sort -u |
    while IFS= read -r f; do [ -f "$f" ] && echo "$f"; done
}

# Issue files outside a docs/issues/ folder: the retired root issues/ and
# <project>/issues/ folders (ISS-001).
stray_files() {
  git ls-files --cached --others --exclude-standard -- 'issues/*.md' '*/issues/*.md' |
    grep -E '(^|/)issues/(human/)?[^/]+\.md$' |
    grep -Ev '(^|/)docs/issues/(human/)?[^/]+\.md$' |
    grep -v '/fixtures/' | sort -u |
    while IFS= read -r f; do [ -f "$f" ] && echo "$f"; done
}

# Prints one tab-separated record per file:
# path, kind, priority, tags, epic, order, added, unblocks, first body line
# (headings skipped), runbook, and which of the three human task sections
# are present (r, w and b for Requirement, Why and Runbook).
records() {
  issue_files | while IFS= read -r f; do
    awk -v file="$f" '
      function val(s) {
        sub(/^[^:]*:[ \t]*/, "", s)
        if (s ~ /^".*"$/) { s = substr(s, 2, length(s) - 2); gsub(/\\"/, "\"", s) }
        gsub(/\t/, " ", s)
        return s
      }
      NR == 1 { if ($0 != "---") { bad = 1; exit } fm = 1; next }
      fm && $0 == "---" { fm = 0; next }
      fm {
        key = $0; sub(/:.*/, "", key)
        f[key] = val($0)
        next
      }
      !fm && /^## Requirement[ \t]*$/ { sections = sections "r" }
      !fm && /^## Why it can.t be automated[ \t]*$/ { sections = sections "w" }
      !fm && /^## Runbook[ \t]*$/ { sections = sections "b" }
      !fm && body == "" && $0 != "" && $0 !~ /^#/ { body = $0 }
      END {
        if (bad || fm) { printf "%s\037BAD\n", file; exit }
        gsub(/\t/, " ", body)
        printf "%s\037%s\037%s\037%s\037%s\037%s\037%s\037%s\037%s\037%s\037%s\n", file, f["kind"], f["priority"], f["tags"], f["epic"], f["order"], f["added"], f["unblocks"], body, f["runbook"], sections
      }' "$f"
  done
}

check() {
  local errors=0
  err() { echo "error[$1] $2"; errors=$((errors + 1)); }
  while IFS= read -r f; do
    err ISS-001 "$f: ISSUES.md files are retired; make one file per issue in an issues/ folder"
  done < <(git ls-files -- 'ISSUES.md' '*/ISSUES.md' 'HUMAN_ISSUES.md')
  while IFS= read -r f; do
    err ISS-001 "$f: issue files live in their project's docs/issues/ folder (repo-wide work in standards/docs/issues/), not in an issues/ folder"
  done < <(stray_files)
  while IFS=$'\037' read -r path kind priority tags _epic order added _unblocks body runbook sections; do
    if [ "$kind" = BAD ]; then
      err ISS-002 "$path: must start with front matter between two '---' lines"
      continue
    fi
    [[ "$added" =~ ^[0-9]{4}-[0-9]{2}-[0-9]{2}$ ]] || err ISS-002 "$path: 'added' must be a YYYY-MM-DD date"
    [ -n "$body" ] || err ISS-002 "$path: says nothing after the front matter"
    if [[ "$path" == */docs/issues/human/* ]]; then
      [[ "$sections" == *r* ]] || err ISS-004 "$path: needs a '## Requirement' section: what to do and what it unblocks"
      [[ "$sections" == *w* ]] || err ISS-004 "$path: needs a '## Why it can't be automated' section"
      [[ "$sections" == *b* ]] || err ISS-004 "$path: needs a '## Runbook' section linking its runbook"
      if [ -z "$runbook" ]; then
        err ISS-004 "$path: needs 'runbook: standards/docs/runbooks/<name>.md' in its front matter"
      elif [[ "$runbook" != standards/docs/runbooks/*.md ]]; then
        err ISS-004 "$path: 'runbook' must be a file in standards/docs/runbooks/ (from the repo root)"
      elif [ ! -f "$runbook" ]; then
        err ISS-004 "$path: runbook $runbook doesn't exist"
      fi
      continue
    fi
    [[ "$kind" =~ ^(todo|fix)$ ]] || err ISS-002 "$path: 'kind' must be todo or fix"
    [[ "$priority" =~ ^(priority|backlog)$ ]] || err ISS-002 "$path: 'priority' must be priority or backlog"
    [[ -z "$order" || "$order" =~ ^[0-9]+$ ]] || err ISS-002 "$path: 'order' must be a whole number"
    local t
    for t in $(echo "$tags" | tr -d '[]' | tr ',' ' '); do
      case "$t" in
        agent-generated | agent-ignore) ;;
        human) err ISS-003 "$path: tasks for a person go in the project's docs/issues/human/" ;;
        *) err ISS-002 "$path: unknown tag '$t' (agent-generated or agent-ignore)" ;;
      esac
    done
  done < <(records)
  if [ "$errors" -gt 0 ]; then
    echo "issues: $errors error(s)"
    return 1
  fi
}

case "$mode" in
  --check) check ;;
  --human)
    records | awk -F'\037' '$1 ~ /\/docs\/issues\/human\// { print $7 "\037" $0 }' | sort |
      while IFS=$'\037' read -r _ path _ _ _ _ _ added unblocks body runbook _; do
        echo "- $added $path: $body${unblocks:+ (Unblocks $unblocks.)}${runbook:+ Runbook: $runbook}"
      done
    ;;
  list | --list)
    records | awk -F'\037' '$1 !~ /\/docs\/issues\/human\// && $2 != "BAD"' |
      awk -F'\037' 'BEGIN { OFS = "\037" } {
        dir = $1; sub(/\/docs\/issues\/[^\/]*$/, "", dir)
        rank = ($3 == "priority") ? 0 : 1
        epic = $5; sub(/ \/ .*/, "", epic)
        printf "%d\037%s\037%s\037%08d\037%s\n", rank, dir, epic, ($6 == "" ? 99999999 : $6), $0
      }' | sort -t$'\037' -k1,1n -k2,2 -k3,3 -k4,4n -k5,5 |
      while IFS=$'\037' read -r _ dir _ _ path kind priority tags epic _ _ _ body _ _; do
        extra=${tags:+ $tags}
        echo "- [$priority] $(echo "$kind" | tr a-z A-Z)$extra $path${epic:+ ($epic)}: $body"
      done
    ;;
  *)
    echo "usage: issues.sh [--human | --check]" >&2
    exit 2
    ;;
esac
