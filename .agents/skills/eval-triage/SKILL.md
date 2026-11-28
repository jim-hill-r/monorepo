---
name: eval-triage
description: Make the eval calls the daily evals-triage workflow can't - rollback advice for a deploy that failed its production evals, and an eval Jim disputes - as issue files and one PR. Use on demand, when a FAILING eval issue names a cd or chaos failure, when Jim says an eval is wrong, or when asked to triage the evals by hand.
---

# Eval triage (judgment calls)

Read `AGENTS.md` first. The daily, free `evals-triage.yml` workflow already
turns every requirement that isn't green into an issue file (`FIX` for a
failing product, `TODO` for a broken, stale or missing eval); see "Triage"
in `standards/docs/evals.md`. Never redo its work, and never run this skill
as a daily Claude task: a quiet day must cost nothing (Jim, 2026-11-27).

This skill covers only the two calls that need judgment. It adds or edits
issue files and opens at most one PR. It never fixes a product or an eval
itself, never reruns a workflow, and never rolls a deploy back: it says how,
and a person decides.

## 1. Gather

Work on the latest `main`, from the repo root.

1. Fetch the results branch into a scratch folder (never check it out in
   the repo, and never merge it):

   ```bash
   git fetch --quiet --depth=1 origin \
     "+refs/heads/evals-results:refs/remotes/origin/evals-results"
   results=$(mktemp -d)
   git archive origin/evals-results | tar -x -C "$results"
   ```

2. The latest result of every eval in each stage (results are appended in
   time order, so the last line wins):

   ```bash
   cat "$results"/results/*/*.jsonl | jq -s '
     group_by([.project, .eval, .stage]) | map(last)' > "$results/latest.json"
   ```

3. The current verdicts, when you need them:
   `bash standards/scripts/evals.sh verdicts --project <p>`.

## 2. A deploy that failed its evals

When: an `eval-<id>-failing.md` issue (from the workflow) or a red deploy
job shows a `must` eval failing in `cd` or `chaos`. The site is already
live with the bad build, so the fix may be slower than a rollback.

1. From the result files (not `latest.json`), find that project's newest
   `commit` whose `cd` results all passed. That is the last good build.
2. Check the failure is the product's, not the eval's: read the result's
   `note` and `evidence` run. If the eval itself is wrong, treat it as in
   step 3 instead.
3. Add a `## Rollback` section to the failing issue file, or put it in the
   PR body: run **Push Main CI/CD** by hand (Actions tab,
   `workflow_dispatch`) with `project` set to the project directory and
   `ref` set to that commit, as in "Deploys" in `standards/docs/mise.md`.
   Say which deploy failed (the result's `evidence` run and `commit`) and
   what a rollback would undo (the commits between the two). If no good
   commit is in the results, say so and point to the project's last green
   Push Main run.
4. Label the PR `needs-human`: only a person chooses to redeploy an older
   build.

## 3. An eval Jim disputes

When: Jim says an eval is wrong (in a thread, a review, or pane feedback
once it exists), or an eval keeps failing while the product plainly works.

1. Quote his words, and say which eval (`id`, project, stage) and what it
   checks today (its entry in the project's `evals.toml`).
2. Write a `TODO` with the `write-issue` skill in the project's `docs/issues/`,
   `eval-<eval-id>-disputed.md`, unless an open issue already names the
   eval. Say what the eval should check instead, or that it should be
   retired, and that its requirement stays as it is unless Jim changes it
   (requirement changes need his approval).
3. If the workflow already filed a `FIX` for that requirement because of
   this eval, edit that issue to point to the new `TODO` rather than
   deleting it: the verdict turns green once the eval is fixed.

## 4. Deliver

- If nothing needed a call, stop and say so in one line.
- Otherwise open one PR with the `open-pull-request` skill, titled
  `Eval triage: <what>, <yyyy-mm-dd>`, changing only issue files. Its body
  says what it found and, for a failed deploy, the rollback above.

## Notes

- Results are files on the `evals-results` branch (decision 0003); this
  skill reads them and writes nothing there.
