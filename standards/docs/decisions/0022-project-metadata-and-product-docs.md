# 0022. Project metadata in one file per project, and a mission, vision and roadmaps in every product

- Status: Proposed
- Date: 2026-11-28
- Asked by: Jim
- Plan: [2026-11-product-strategy.md](../plans/2026-11-product-strategy.md)
- Builds on: [0006](0006-one-file-per-issue.md) (one file per thing),
  [0017](0017-user-docs.md) (required docs with a shrinking baseline) and
  [0019](0019-review-roles.md) (tiers and review roles)

## Context

Jim (2026-11-28): "Each product project should have a mission, vision,
product roadmap and technical roadmap. Lets make a plan to include that in
each product project. How are we storing metadata about a project (for now
what tier it is but later more might become relevant)?"

No product has a written mission or roadmap; direction lives in READMEs,
plans and chat. And there is no single store of project metadata: the tier
is only a prose table in 0019, the on-hold list is a bullet list in
`issues/README.md`, and other facts are spread over a dozen files (the
table in the plan's "Question 1").

## Decision

1. **Project metadata lives in `<project>/docs/project.toml`**, one per
   top-level project, starting with `tier` and `status` (`active` or
   `on-hold`). New fields are added with the check that reads them. A
   `projects` module in `chores` is the only reader; PRJ-001 (file missing)
   and PRJ-002 (unknown key or value) run in the audit. The on-hold list
   and 0019's tier list move here; `standards/reviewers.toml` keeps only
   the role grants. Adding a `project.toml` or changing its `tier` counts
   as a `software-development-platform` change in the review roles check.
2. **Every `product` project has `docs/mission.md`** (`## Mission`,
   `## Vision`) **and `docs/roadmap.md`** (`## Product roadmap`,
   `## Technical roadmap`, each with `### Now`, `### Next`, `### Later`).
   Roadmap items link the issue files or epics that do the work.
3. **`chores product-docs` checks them** (DOC-009 to DOC-012) against the
   shrinking baseline `standards/product-docs.toml`, and a
   `write-mission-and-roadmap.md` issue in each active product writes
   them.
4. **Jim owns the mission and vision.** A change to `docs/mission.md` is a
   `requirements` change in the review roles check, so it needs the
   `requirements-reviewer`. Roadmaps are ordinary product changes that
   agents keep current as they close issues.

## Options considered for the metadata store

| Option | For | Against |
| --- | --- | --- |
| `<project>/docs/project.toml` (chosen, pending Jim) | Parallel PRs never conflict; lives with `cost.toml` and `threat.toml`; a new project adds its own file | A tier change must be detected per file to keep strict review |
| One `standards/projects.toml` | Every project on one screen; any edit is already a strict change | Every new project or field edits one shared file |
| Tiers in `standards/reviewers.toml` (0019 as written) | No change | Mixes who-may-approve with what-a-project-is; other fields still have no home |
| `[package.metadata]` in `Cargo.toml` or a table in `mise.toml` | No new file | Not every project is Rust; mise rejects unknown keys in its config |

## Consequences

- Cost: $0, no vendor, no service; about four small agent PRs and ten
  short documents against the Claude Code budget.
- Decision 0019 changes in one place: tiers are read from `project.toml`,
  not listed in `reviewers.toml`. Review roles phase 1 is updated to match.
- pane, the deep review, `next-task` and the eval engine can read a
  project's tier, status, mission and roadmap from fixed paths.
