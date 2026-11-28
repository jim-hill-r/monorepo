# Baked skills: deterministic scripts for routine agent work

- Status: proposed (Jim asked for it, 2026-11-27)
- Date: 2026-11-27
- Project: [`chores`](../../../chores/README.md)

## Goal

Jim, 2026-11-27: "let's start adding some agent skills that leverage
deterministic rust scripts for work we are repeatedly doing in an effort to
save tokens. Our goal is to only use tokens for novel work and have many
baked skills and scripts for doing routine work."

Every agent thread spends tokens re-deriving the same answers: reading the
issue list and README to pick a task, rebuilding the PR description shape,
re-reading the issue format before writing a file. When the answer follows
from rules already written down, a script gives it in one call and the
model only does the part that needs judgment.

## What threads repeat most

Counted from the project's thread list on 2026-11-27 (177 threads in about
two days) and from what the skills ask each thread to do. Token figures are
estimates of what a thread spends on the step today, from reading the files
the step needs, not measurements.

| Rank | Routine work | How often | Tokens per run today (est.) | With a script | Status |
| --- | --- | --- | --- | --- | --- |
| 1 | Pick the next issue (`start-a-new-task` step 1, and the coordinator choosing one project per new thread) | Nearly every thread: about 60 a day, most titled "Advance X's next task" | 15k to 30k: the issue list, `standards/docs/issues/README.md`, folder READMEs and several issue files | About 1k: one pick with its text and epic notes | Built: `chores next-task` |
| 2 | Write and check the PR description (attribution lines, Attention tags, the What changed, Why, Cost changes and Risk changes summaries with their word limits, Details, footer; Jim, 2026-11-27, replacing the 75-word summary) | Every PR: about 50 a day | 5k to 10k: the skill, `pull-requests.md`, the template, and a fix after the PR-001 warning | About 1k | Built: `chores pr-body` |
| 2b | Attention tags at the top of each PR, from a closed set: a requirement change (red for a `must`), a cost change and a threat/risk change (Jim, 2026-11-27) | Every PR | Jim reading the file list to judge how closely to look | One line from the diff | Built: `chores badges`, called by `pr-body` |
| 3 | Hourly credit-window check (usage math, counting threads) | 24 runs a day | 10k to 20k: the decision's rule, the events, the arithmetic | About 2k: fetch events and threads, one call | Next, once [PR 819](https://github.com/jimhillr/monorepo/pull/819) (decision 0013) merges: [issue](../../../chores/docs/issues/credit-window-command.md) |
| 4 | Write issue files and human tasks with their runbooks | About 30 a day | 3k to 5k: the format, the runbook template, an audit retry | Under 1k | Built: `chores new-issue` |
| 5 | Session setup in a cloud sandbox (mise from npm, hooks, `MISE_OFFLINE`) | Every thread | 3k to 8k, more when an install fails | Under 1k | Next: [issue](../../../chores/docs/issues/session-setup-command.md) |
| 6 | Watch CI and decide "ready" (checks green apart from ones red on `main`, then ready and auto-merge) | Every PR, several wakes each | 2k to 5k a wake | The model still fetches checks through GitHub tools; a verdict command saves the reasoning | Later: [issue](../../../chores/docs/issues/pr-ready-verdict-command.md) |
| 7 | `needs-human` labelling | Some PRs | Small: the Cost summary job already labels cost and vendor changes | Little left to save | Not planned |
| 8 | Rename sweeps (`monorepo` to `jimhillr`) | A few threads | Large once, then rare | A guarded find-and-replace | Not planned: one-off |
| 9 | `threat.toml` and header checks | Some threads | Already a script (`threat check`) | Nothing to add | Not planned |

## Design

- **One Rust CLI, `chores`**, a new top-level project, so its commands
  share parsing (issue front matter, the README project lists) and its
  tests. No existing tools crate fits: `cost`, `evals` and `threat` each
  own one domain, and `monorepo/workflow_tests` is being renamed.
  The name follows the naming rule (never `monorepo`).
- **Deterministic and offline.** Commands read only the working tree and
  `git ls-files`, never the network. GitHub calls stay with the model's
  GitHub tools (the sandboxes' `gh` has no usable token), so a command
  prints exactly what to send rather than sending it.
- **Skills call it and keep the judgment.** Each command is run from an
  existing or new skill with
  `cargo run --quiet --manifest-path chores/Cargo.toml -- <command>`
  (cargo is in every sandbox; mise often isn't). The skill says what the
  model still decides: whether the picked issue is really workable, what
  the summary says, what a human task asks.
- **Same rules, one source.** The commands implement rules already in the
  standards (`issue-management.md`, `pull-requests.md`, the
  `start-a-new-task` order). Where a shell check exists (`issues.sh
  --check`, `pr-summary-check.sh`), the command matches it and the audit
  stays the gate.

## Tradeoffs

- A first `cargo run` compiles the crate (about a minute in a sandbox).
  That is still far cheaper in tokens than the reading it replaces, and
  later runs in the same session are instant.
- The rules now live in prose and in code. Tests pin the code to the
  examples in the standards, and a skill that disagrees with its command
  should be fixed in the same PR.
- One PR for the first three commands: this thread may push only one
  branch, so each command and its skill change is one commit instead of
  one PR.

## Cost

$0. The commands run in sandboxes and CI on the free GitHub Actions
minutes the hooks already use. Building them is about one Claude Code
thread, against the $25 a month budget, and each saves an estimated 20k to
40k tokens per thread after that.
