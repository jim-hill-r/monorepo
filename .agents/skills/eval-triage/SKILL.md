---
name: eval-triage
description: Read the eval engine's verdicts and results on the evals-results branch and turn what broke into issue files (a FIX for each must requirement not met, a TODO for each eval in error) plus rollback instructions for a failed deploy, in one draft PR. Use for the daily eval triage scheduled task, or when asked to triage the evals.
---

# Eval triage

Turn the eval engine's latest verdicts into work an agent can pick up. Read
`AGENTS.md` first. The design is under "Triage" in
`standards/docs/plans/2026-11-eval-engine.md`, and the formats of
requirements, evals, results and verdicts are in `standards/docs/evals.md`.

This skill only adds issue files, through at most one PR. It never fixes a
product or an eval itself, never reruns a workflow, and never rolls a deploy
back: it says how, and a person or a later agent does it.

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

   If the branch doesn't exist yet, there is nothing to triage: stop.
2. Compute the verdicts from it with the requirements and evals on `main`
   (fresher than the `verdicts.json` the last store wrote):

   ```bash
   cargo run -q --manifest-path evals/Cargo.toml -p evals_cli -- \
     verdicts --root . --dir "$results" --json > "$results/now.json"
   ```

3. List the latest result of every eval in each stage (the last line wins,
   since results are appended in time order):

   ```bash
   cat "$results"/results/*/*.jsonl | jq -s '
     group_by([.project, .eval, .stage]) | map(last)' > "$results/latest.json"
   ```

## 2. Decide

Work through three cases. Before adding any file, search the open issues
(`grep -rl` over every `issues/` folder, and open PRs whose title starts
with "Eval triage") so a problem gets one issue, not one a day.

1. **A requirement not met.** Every requirement with verdict `not-met` in
   `now.json` (only `must` requirements get it; a failing `should` or
   `could` is `at-risk`, which this skill leaves alone). Skip it when an
   open issue file with `kind: fix` already names its ID. Otherwise add
   `<project>/issues/<id-lowercase>-not-met.md`:

   ```markdown
   ---
   kind: fix
   priority: priority
   tags: [agent-generated]
   added: <today>
   ---

   COOK-001 is not met in production: There is a recipe for every day of the year.

   Failing evals (from the verdict): `recipe-pages` failed in cd at 2026-11-28T10:31Z.
   Evidence: <the `evidence` link of each failing result>, commit <short sha>.

   The requirement is in [requirements.md](../docs/requirements.md#cook-001)
   and its evals in [evals.toml](../evals.toml). Fix the product, not the
   eval, unless the eval is wrong; then say why in the PR. The next eval
   run shows whether the fix worked.
   ```

   Quote the verdict's `reasons` as they are. For a sub-project's
   requirement, the file still goes in the top-level project's `issues/`,
   where its `requirements.md` is.
2. **An eval in error.** Every entry in `latest.json` with
   `"status": "error"`: the eval itself is broken (it matched no test, or
   an `http` eval had no deploy URL), not the product. Skip it when an open
   issue file already names the eval's `id`. Otherwise add
   `<project>/issues/eval-<eval-id>-error.md` with `kind: todo`,
   `priority: backlog`, `tags: [agent-generated]`, saying which eval, which
   stage, the result's `note` and `evidence`, and that the fix is in
   `evals.toml` or the tests it should match. One file per eval, however
   many stages it errored in.

   When Jim disputes an eval (pane feedback, a later phase), treat it the
   same way: a `TODO` to fix the eval, quoting his words.
3. **A deploy that failed its evals.** Every entry in `latest.json` with
   `"stage": "cd"` (or `"chaos"`) and `"status": "fail"` whose requirement
   is a `must`. The site is already live with the bad build. Find the
   last good commit: the newest `commit` of that project whose `cd`
   results all passed (from the result files, not `latest.json`). Put the
   rollback in the PR body (step 3), not in an issue: run **Push Main
   CI/CD** by hand (Actions tab, `workflow_dispatch`) with `project` set to
   the project directory and `ref` set to that commit, as in "Deploys" in
   `standards/docs/mise.md`. If no good commit is in the results, say so
   and point to the project's last green Push Main run. The `not met`
   issue from case 1 covers the fix itself.

Requirements that are `unknown` or `at-risk` need nothing from this skill:
`unknown` just means production evals haven't run yet, and `at-risk` shows
in pane and the verdicts for a person to judge.

## 3. Deliver

- If nothing qualified (no new file and no failed deploy), stop. Open no PR
  and post nothing beyond one line in the run's own thread.
- Otherwise open one PR with the `open-pull-request` skill, titled
  `Eval triage, <yyyy-mm-dd>`, changing only the new issue files. Its body
  says, in a few lines: how many requirements are not met and in which
  projects, the issues it added, the evals in error, and for each failed
  deploy which deploy it was (the result's `evidence` run and `commit`)
  and the rollback above. It labels the PR `needs-human` only when a
  rollback is suggested, since only a person should choose to redeploy an
  older build.
- Post one short message saying what it found and linking the PR.

## Notes

- Results are files on the `evals-results` branch (decision 0003); this
  skill reads them and writes nothing there.
- The scheduled task that runs this skill daily is
  `standards/docs/scheduled-tasks/eval-triage.md`.
