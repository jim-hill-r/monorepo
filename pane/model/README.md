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

The `costs` module puts every project's `docs/cost.toml` together with the
radar's monthly budgets and the free limits: `build_costs` gives `Costs`,
with each project's services, plans, expected usage, cost and revenue lines
and this month's money, account-wide expected usage against each limit's
ceiling, and each vendor's spend this month against its radar budget.
`pane_snapshot` fills `Snapshot.costs` with it. `build_history` adds
twelve `MonthMoney` rows (each project's `ProjectMoney` for the month,
with the shared costs from `shared_cents` split by that month's merged
PRs), and `MonthMoney::totals` sums a month for the Money page.

## Review queue

The `queue` module holds the Approve page's rules from [decision
0014](../../standards/docs/decisions/0014-faster-pr-approvals.md). It parses
GitHub's open pull requests and a search for green PRs that need a review,
and builds a `Queue`: PRs to approve (clear, then orange), PRs labelled red
that need a decision, and stacks. `summary` cuts a PR body to its executive
summary and `attention` turns labels into a red, orange or clear badge.

The `acks` module is the Acknowledge page's post-merge review: it parses
GitHub's search for merged PRs, keeps the acknowledged numbers in an
`AckStore`, lists the rest newest first, and builds the "Flag for
follow-up" link, a GitHub new-file URL prefilled with a `fix` issue file in
the PR's project.

## What's in it

| Type | Holds |
| --- | --- |
| `Snapshot` | Everything read from one checkout of the repo: the format version, commit, time taken, and the lists below. |
| `Project` | One entry from `[monorepo].config_roots`: path, type, `DEPLOY_URL`, README summary, priority or on-hold status. |
| `WorkItem` | One `TODO` or `FIX`, from an issue file or a code comment, with its file, line, section, epic, tags and the date it was added. |
| `HumanIssue` | One task file in a project's `docs/issues/human/`, with its project, added date and what it unblocks. |
| `Deliverable` | A live site, plan, decision, retro, review deck, release or UAT report, with its download links and, for documents, its Markdown `body`. New fields default, so the format version stays 1. |
| `Review` | An executive review deck and its scorecard rows. |
| `Retro` | A retro and its actions. |
| `Costs` | Every project's `docs/cost.toml`, account-wide usage against the free limits and vendor spend against the radar budgets, for the month the snapshot was taken. Defaults to empty, so the format version stays 1. |
| `Signal`, `Health` | One traffic light (green, amber, red or grey) with its reason and evidence links. |
| `audit::Audit`, `audit::parse` | The repo audit's findings, read from its output (`level[CODE] <subject> ...` lines and a closing `audit:` line); each finding belongs to the longest config root that contains its subject. `Snapshot.audit` is `None` without them. |
| `signals::project_signals` | A project's Review, Retros, Open issues and Standards lights, worked out from the snapshot alone. |
| `health::Thresholds`, `health::Rollup` | The thresholds from `pane/health.toml`, and a row's worst-of light with its coverage. |

## Format version

`SNAPSHOT_VERSION` is written into every snapshot. Bump it whenever a change
would stop an older reader from parsing the JSON (a renamed or removed field,
a new required field). `Snapshot::from_json` refuses a snapshot written with a
different version, so the web app can say the data is out of date instead of
showing it wrong.

## Health rules

The `health` module holds the traffic-light rules from Phase 4 of the
[review dashboard plan](../docs/plans/2026-11-review-dashboard.md): one pure
function per project signal (`build`, `deploy_and_uat`, `observability`,
`review`, `retros`, `open_issues`, `standards`, `pull_requests`) and system
signal (`nightly`, `push_main`, `free_tiers`, `pr_queue`, `human_queue`,
`agent_tasks`, `review_rotation`, `security_scans`), and `roll_up`, which
gives a row the worst of its non-grey signals and a coverage count such as
"5 of 7 signals". The functions take ages in whole days and counts that the
caller has already worked out, so they need no clock and compile to wasm.

Their thresholds are the `Thresholds` type, read from
[`pane/health.toml`](../health.toml) by `pane_snapshot`
(`parse::health_thresholds`). Every age rule means "more than N days". A
missing key takes the plan's starting value and an unknown key is an error,
so tune a threshold with a one-line PR to that file.

## Snapshot signals

The `signals` module applies those rules to a `Snapshot`, using the
thresholds the snapshot carries (`Snapshot::thresholds`, from
`pane/health.toml`). `project_signals(&snapshot, "pane")` gives the four
signals the snapshot can light on its own:

| Signal | Worked out from |
| --- | --- |
| Review | The newest deck in the rotation for the project's top-level folder (`cookbook` for `cookbook/web`), its scorecard rows rated Poor, and a rotation of one week per top-level project that isn't on hold. |
| Retros | The project's newest retro; each open action has been open since the retro's date. |
| Open issues | The project's `FIX` items, aged from `WorkItem::added`, and the human issues in the project's own `docs/issues/human/` or whose `unblocks` names the project as a whole word. |
| Standards | The project's own audit errors in `Snapshot::audit` (red, each one linked as evidence), then its open code-review findings (agent-generated `TODO`s outside any epic). Grey when the snapshot was built without the audit, or the audit didn't finish. |

Ages are whole days from a date to `taken_at` (`signals::days_between`), so
the lights depend only on the snapshot. Each signal links its evidence (the
deck, the retro, each `FIX` line) on GitHub. Build, Deploy and UAT, and Pull
requests need live GitHub data and are added in a later phase.
