# pane_model

The shared data types for [pane](../), the review dashboard. `pane_snapshot`
writes a `Snapshot` to `snapshot.json`, and `pane_web` reads it back, so both
sides agree on one set of serde types defined here. It is named for what it
holds: pane's data model.

## What's in it

| Type | Holds |
| --- | --- |
| `Snapshot` | Everything read from one checkout of the repo: the format version, commit, time taken, and the lists below. |
| `Project` | One entry from `[monorepo].config_roots`: path, type, `DEPLOY_URL`, README summary, priority or on-hold status. |
| `WorkItem` | One `TODO` or `FIX`, from an `ISSUES.md` line or a code comment, with its file, line, section, epic and tags. |
| `HumanIssue` | One `TODO (human)` line in `HUMAN_ISSUES.md`, with its added date and what it unblocks. |
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
