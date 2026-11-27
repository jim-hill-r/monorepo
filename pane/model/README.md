# pane_model

The shared data types for [pane](../), the review dashboard. `pane_snapshot`
writes a `Snapshot` to `snapshot.json`, and `pane_web` reads it back, so both
sides agree on one set of serde types defined here. It is named for what it
holds: pane's data model.

## Money

The `money` module holds the revenue and cost rules from "Revenue and cost"
in the [review dashboard plan](../docs/plans/2026-11-review-dashboard.md),
built on [`cost_model`](../../cost/model) so pane and `cost check` read
`docs/cost.toml` the same way. `ProjectMoney` is one project's month:
revenue, direct cost, share of shared costs and net, in whole US cents.
`monthly_revenue`, `direct_cost` (yearly costs spread over twelve months, a
once cost only in its month) and `net` work it out from a project's
`docs/cost.toml`, and `split_shared_costs` splits a shared cost such as Claude
Code across projects by their share of the month's merged PRs. A PR touching
two projects counts half to each, a repo-wide PR counts for none, and the
shares always add up to the whole cost. `month_money` puts them together for
the Money page.

## Review queue

The `queue` module holds the Approve page's rules from [decision
0014](../../standards/docs/decisions/0014-faster-pr-approvals.md). It parses
GitHub's open pull requests and a search for green PRs that need a review,
and builds a `Queue`: PRs to approve (clear, then orange), PRs labelled red
that need a decision, and stacks. `summary` cuts a PR body to its executive
summary and `attention` turns labels into a red, orange or clear badge.

## What's in it

| Type | Holds |
| --- | --- |
| `Snapshot` | Everything read from one checkout of the repo: the format version, commit, time taken, and the lists below. |
| `Project` | One entry from `[monorepo].config_roots`: path, type, `DEPLOY_URL`, README summary, priority or on-hold status. |
| `WorkItem` | One `TODO` or `FIX`, from an issue file or a code comment, with its file, line, section, epic and tags. |
| `HumanIssue` | One task file in `issues/human/`, with its added date and what it unblocks. |
| `Deliverable` | A live site, plan, decision, retro, review deck, release or UAT report, with its download links and, for documents, its Markdown `body`. New fields default, so the format version stays 1. |
| `Review` | An executive review deck and its scorecard rows. |
| `Retro` | A retro and its actions. |
| `Signal`, `Health` | One traffic light (green, amber, red or grey) with its reason and evidence links. |

## Format version

`SNAPSHOT_VERSION` is written into every snapshot. Bump it whenever a change
would stop an older reader from parsing the JSON (a renamed or removed field,
a new required field). `Snapshot::from_json` refuses a snapshot written with a
different version, so the web app can say the data is out of date instead of
showing it wrong.

The health rules (thresholds and roll-ups) are added here in Phase 4 of the
[review dashboard plan](../docs/plans/2026-11-review-dashboard.md).
