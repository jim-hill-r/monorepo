#!/usr/bin/env bash
# Repo-wide security guards run by //:audit (standards/docs/security.md).
# Reads only repo files, so it takes milliseconds. Exits non-zero on any error.
#
#   SEC-002  .gitignore ignores the secret files (.env, .dev.vars, *.pem,
#            *.key, *.p12, *.jks, *.keystore), and no tracked file is one.
#   SEC-006  Every tracked Cargo.lock and package-lock.json has a
#            .github/dependabot.yml entry for its directory, and every entry's
#            directory exists (and, for cargo and npm, holds that lockfile).
#   SEC-007  CI's only git pushes are evals.sh's push of eval results to the
#            evals-results data branch, coverage.sh's push of coverage
#            results to the coverage-results data branch, feedback.sh's
#            push of fan feedback to feedback-data, and evals.sh triage's
#            push (and delete) of the evals-triage branch: no other `git push`
#            in a workflow, shared script or shared task file, and never to
#            main.
#
# SEC-001 (gitleaks) is secrets-check.sh, run by the hooks and PR CI.
set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

errors=0
err() { echo "error[$1] $2"; errors=$((errors + 1)); }

# SEC-002: a sample path for each kind of secret file, at the root and deep in
# a project, must be ignored.
secret_files=(.env .env.local .dev.vars .dev.vars.production key.pem tls.key cert.p12 release.jks release.keystore)
for f in "${secret_files[@]}"; do
  for path in "$f" "some_project/web/$f"; do
    git check-ignore -q --no-index -- "$path" \
      || err SEC-002 ".gitignore does not ignore $path; add a pattern for it"
  done
done
# A public example (.env.example) is fine to commit.
if git check-ignore -q --no-index -- some_project/.dev.vars.example; then
  err SEC-002 ".gitignore ignores .dev.vars.example; keep the !**/*.example exception"
fi
while IFS= read -r f; do
  [ -n "$f" ] || continue
  case "$f" in *.example) continue ;; esac
  err SEC-002 "$f is tracked but is a secret file; remove it (git rm --cached) and rotate what it held"
done < <(git ls-files -- ':(glob)**/.env' ':(glob)**/.env.*' ':(glob)**/.dev.vars' \
  ':(glob)**/.dev.vars.*' '*.pem' '*.key' '*.p12' '*.pfx' '*.jks' '*.keystore')

# SEC-006: dependabot.yml covers every lockfile. Entries are read as
# "ecosystem directory" pairs: each `directory:` line belongs to the
# `package-ecosystem:` line above it.
dependabot=.github/dependabot.yml
if [ ! -f "$dependabot" ]; then
  err SEC-006 "$dependabot is missing"
else
  entries=$(awk '
    /^[[:space:]]*-[[:space:]]*package-ecosystem:/ { eco = $NF; gsub(/["\047]/, "", eco) }
    /^[[:space:]]*directory:/ { dir = $NF; gsub(/["\047]/, "", dir); print eco, dir }
  ' "$dependabot")
  while read -r eco dir; do
    [ -n "$eco" ] || continue
    rel=${dir#/}
    rel=${rel:-.}
    if [ ! -d "$rel" ]; then
      err SEC-006 "$dependabot: $eco entry for $dir, which does not exist; remove it"
      continue
    fi
    case "$eco" in
      cargo) lock=Cargo.lock ;;
      npm) lock=package-lock.json ;;
      *) continue ;;
    esac
    git ls-files --error-unmatch -- "$rel/$lock" >/dev/null 2>&1 \
      || err SEC-006 "$dependabot: $eco entry for $dir, which has no tracked $lock (for a Cargo workspace, use the workspace root)"
  done <<<"$entries"
  while IFS= read -r lock; do
    d=$(dirname "$lock")
    case "$(basename "$lock")" in
      Cargo.lock) eco=cargo ;;
      package-lock.json) eco=npm ;;
    esac
    want="$eco /${d#.}"
    [ "$d" = . ] && want="$eco /"
    grep -qxF "$want" <<<"$entries" \
      || err SEC-006 "$lock has no $eco entry in $dependabot; add one for /$d"
  done < <(git ls-files -- ':(glob)**/Cargo.lock' ':(glob)**/package-lock.json')
fi

# SEC-007: the eval workflows push only to evals-results, the coverage
# workflow only to coverage-results, the feedback export only to
# feedback-data, and eval triage only to evals-triage (a branch PR triage
# opens a pull request from, so main still changes only through review). Comment lines are skipped; every other
# line running `git ... push` must be evals.sh's or coverage.sh's one push,
# which names its target ref in full. This file holds the patterns.
allowed='git -C "$work" push --quiet origin HEAD:refs/heads/evals-results'
allowed_coverage='git -C "$work" push --quiet origin HEAD:refs/heads/coverage-results'
allowed_feedback='git -C "$work" push --quiet origin HEAD:refs/heads/feedback-data'
allowed_triage='git -C "$work" push --quiet --force origin HEAD:refs/heads/evals-triage'
allowed_triage_delete='git -C "$work" push --quiet origin --delete refs/heads/evals-triage'
while IFS=: read -r file line text; do
  [ -n "$file" ] || continue
  [[ "$text" =~ ^[[:space:]]*# ]] && continue
  [ "$file" = standards/scripts/security-check.sh ] && continue
  if [ "$file" = standards/scripts/evals.sh ] && [[ "$text" == *"$allowed"* ]]; then
    continue
  fi
  if [ "$file" = standards/scripts/coverage.sh ] && [[ "$text" == *"$allowed_coverage"* ]]; then
    continue
  fi
  if [ "$file" = standards/scripts/feedback.sh ] && [[ "$text" == *"$allowed_feedback"* ]]; then
    continue
  fi
  if [ "$file" = standards/scripts/evals.sh ] \
    && { [[ "$text" == *"$allowed_triage"* ]] || [[ "$text" == *"$allowed_triage_delete"* ]]; }; then
    continue
  fi
  err SEC-007 "$file:$line runs git push; CI may push only eval results to evals-results (standards/scripts/evals.sh store), coverage results to coverage-results (standards/scripts/coverage.sh store), fan feedback to feedback-data (standards/scripts/feedback.sh store) and triage issue files to evals-triage (standards/scripts/evals.sh triage)"
done < <(git grep -nE 'git( +-[cC] +[^ ]+)* +push( |$)' -- '.github/workflows/*.yml' '.github/workflows/*.yaml' \
  'standards/scripts/*.sh' 'standards/tasks/*.toml' || true)

if [ "$errors" -gt 0 ]; then
  echo "security-check: $errors error(s)"
  exit 1
fi
