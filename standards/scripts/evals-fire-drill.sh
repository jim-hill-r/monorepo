#!/usr/bin/env bash
# Eval fire drill (standards/docs/evals.md#fire-drill): prove that a broken
# production check turns a requirement red and that the next good run turns
# it green again, without touching production or the stored results.
#
#   evals-fire-drill.sh [<project> [<eval-id>]]
#       Defaults: cookbook, recipe-page-live. The eval must be an `http`
#       eval in the project's evals.toml.
#
# Steps, each checked (the drill exits 1 at the first one that doesn't hold):
#   1. Copy the evals-results branch into a temporary folder (read only; the
#      drill never pushes) and note the requirement's verdict.
#   2. Red run: in a temporary copy of the repo, point the eval at a string
#      the live page doesn't have (body_contains), run stage cd against the
#      live site, and check the eval's result is `fail`.
#   3. Store that result in the temporary copy of evals-results and check the
#      requirement's verdict is now `not-met` (`at-risk` for a should or
#      could).
#   4. List what eval triage should file for it (the free triage workflow,
#      issues/eval-triage-free-workflow.md, files it for real).
#   5. Green run: the unchanged eval against the live site, stored the same
#      way, and check the verdict is back to what it was in step 1.
#
# Only GET requests to the live site are made, the same ones a cd run makes.
# The deploy URL is $DEPLOY_URL, or DEPLOY_URL from a mise.toml in the
# project. Writes a report to $GITHUB_STEP_SUMMARY (or standard output).
set -euo pipefail

project=${1:-cookbook}
eval_id=${2:-recipe-page-live}
root=$(git rev-parse --show-toplevel)
summary=${GITHUB_STEP_SUMMARY:-/dev/stdout}
miss="jimhillr-fire-drill-$(date -u +%Y%m%d%H%M%S)-not-on-the-page"

work=$(mktemp -d)
cleanup() {
  git -C "$root" worktree remove --force "$work/root" 2>/dev/null || true
  rm -rf "$work"
}
trap cleanup EXIT

fail() {
  echo "fire drill: FAILED: $*" >&2
  echo "| Result | **Failed:** $* |" >> "$summary"
  exit 1
}

cargo build --quiet --manifest-path "$root/evals/Cargo.toml" -p evals_cli
evals_cli="$root/evals/target/debug/evals"

evals_toml="$root/$project/evals.toml"
[ -f "$evals_toml" ] || fail "$project has no evals.toml"

# The eval's requirement and kind, from evals.toml.
read -r requirement kind < <(python3 - "$evals_toml" "$eval_id" <<'EOF'
import sys, tomllib
with open(sys.argv[1], "rb") as f:
    evals = tomllib.load(f).get("eval", [])
match = [e for e in evals if e.get("id") == sys.argv[2]]
print(f"{match[0]['requirement']} {match[0]['kind']}" if match else "- -")
EOF
)
[ "$requirement" != - ] || fail "$project/evals.toml has no eval $eval_id"
[ "$kind" = http ] || fail "$eval_id is a $kind eval; the drill needs an http one"

deploy_url=${DEPLOY_URL:-$(git -C "$root" ls-files -- "$project/mise.toml" "$project/*/mise.toml" |
  while read -r f; do sed -n 's/^DEPLOY_URL *= *"\(.*\)"/\1/p' "$root/$f"; done | head -1)}
[ -n "$deploy_url" ] || fail "no DEPLOY_URL for $project"

verdict_of() { # <results dir> -> the requirement's verdict in verdicts.json
  jq -r --arg p "$project" --arg r "$requirement" \
    '.projects[] | select(.project == $p) | .requirements[] | select(.requirement == $r) | .verdict' \
    "$1/verdicts.json"
}
result_of() { # <results.jsonl> -> the eval's status
  jq -r --arg e "$eval_id" 'select(.eval == $e) | .status' "$1" | tail -1
}

{
  echo "## Eval fire drill: $project $eval_id ($requirement)"
  echo
  echo "Against $deploy_url. Results went to a temporary copy of \`evals-results\`; nothing was pushed."
  echo
  echo "| Step | Outcome |"
  echo "| --- | --- |"
} >> "$summary"

# 1. Baseline, from a read-only copy of the results branch.
mkdir "$work/results"
if git -C "$root" fetch --quiet --depth=1 origin "+refs/heads/evals-results:refs/remotes/origin/evals-results" 2>/dev/null; then
  git -C "$root" archive origin/evals-results | tar -x -C "$work/results"
fi
"$evals_cli" verdicts --root "$root" --dir "$work/results" --json > "$work/results/verdicts.json"
baseline=$(verdict_of "$work/results")
echo "| 1. Baseline verdict | \`$baseline\` |" >> "$summary"

# 2. Red run: the eval looks for a string that isn't on the page.
git -C "$root" worktree add --quiet --detach "$work/root" HEAD
python3 - "$work/root/$project/evals.toml" "$eval_id" "$miss" <<'EOF'
import re, sys
path, eval_id, miss = sys.argv[1:]
blocks = re.split(r"(?m)^(?=\[\[eval\]\])", open(path).read())
for i, block in enumerate(blocks):
    if re.search(rf'(?m)^id\s*=\s*"{re.escape(eval_id)}"\s*$', block):
        block = re.sub(r"(?m)^body_contains\s*=.*\n", "", block)
        blocks[i] = re.sub(r"(?m)^(kind\s*=\s*\"http\"\s*\n)", rf'\1body_contains = "{miss}"\n', block, count=1)
open(path, "w").write("".join(blocks))
EOF
grep -q "$miss" "$work/root/$project/evals.toml" || fail "could not break $eval_id"
"$evals_cli" run --root "$work/root" --stage cd --project "$project" --deploy-url "$deploy_url" \
  --commit fire-drill-red --out "$work/red.jsonl" --http-attempts 1 || true
red=$(result_of "$work/red.jsonl")
note=$(jq -r --arg e "$eval_id" 'select(.eval == $e) | .note' "$work/red.jsonl" | tail -1)
[ "$red" = fail ] || fail "the broken eval's result was '$red', not fail ($note)"
echo "| 2. Broken eval's result | \`$red\`: $note |" >> "$summary"

# 3. The stored failure turns the requirement red (amber for should/could).
"$evals_cli" store --root "$work/root" --dir "$work/results" --results "$work/red.jsonl"
red_verdict=$(verdict_of "$work/results")
case "$red_verdict" in
  not-met|at-risk) ;;
  *) fail "after the failure the verdict was '$red_verdict', not not-met or at-risk" ;;
esac
echo "| 3. Verdict after the failure | \`$red_verdict\` |" >> "$summary"

# 4. What triage should file.
issue=$(grep -rl -- "$requirement" "$root/$project/issues" 2>/dev/null | head -1 || true)
if [ -n "$issue" ]; then
  triage="already tracked in ${issue#"$root"/}, so nothing new to file"
else
  triage="a \`kind: fix\` issue in \`$project/issues/\` naming $requirement and $eval_id"
fi
echo "| 4. Triage should file | $triage |" >> "$summary"

# 5. Green run: the unchanged eval passes and the verdict recovers.
"$evals_cli" run --root "$root" --stage cd --project "$project" --deploy-url "$deploy_url" \
  --commit fire-drill-green --out "$work/green.jsonl" || true
green=$(result_of "$work/green.jsonl")
[ "$green" = pass ] || fail "the unchanged eval's result was '$green', not pass"
"$evals_cli" store --root "$root" --dir "$work/results" --results "$work/green.jsonl"
green_verdict=$(verdict_of "$work/results")
[ "$green_verdict" = "$baseline" ] || fail "after the good run the verdict was '$green_verdict', not '$baseline' as before"
echo "| 5. Verdict after the good run | \`$green_verdict\` (result \`$green\`) |" >> "$summary"
echo "| Result | **Passed:** red on a broken check, back to \`$baseline\` on the next good run |" >> "$summary"
echo "fire drill: passed ($requirement: $baseline -> $red_verdict -> $green_verdict)"
