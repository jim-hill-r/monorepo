# Plan: a mission, vision and roadmaps for every product, and one home for project metadata

Started 2026-11-28 from Jim's request:

> Each product project should have a mission, vision, product roadmap and
> technical roadmap. Lets make a plan to include that in each product
> project. How are we storing metadata about a project (for now what tier
> it is but later more might become relevant)?

Status: **approved** by Jim (decision card, 2026-11-28), with the
recommended answer under [Decisions](#decisions). The decision record is
[0022](../decisions/0022-project-metadata-and-product-docs.md). The work
items are issue files in the root [issues/](../../../issues/) with
`epic: Product strategy`.

This plan copies the pattern that made user guides required
([decision 0017](../decisions/0017-user-docs.md)): fixed file paths with
fixed `##` sections, a deterministic `chores` check in `mise run //:audit`,
a baseline that only shrinks, and one issue file per project so the normal
agent loop writes the documents.

## Goal

1. **Every product says why it exists and where it is going.** Each
   project in the `product` tier has a mission, a vision, a product roadmap
   and a technical roadmap, at the same paths in every project, so a person,
   an agent or pane can find them without searching.
2. **They stay true.** Roadmaps point at the issue files that do the work,
   so when an issue closes the roadmap has to change with it, and the audit
   catches a roadmap that points at work that no longer exists.
3. **Facts about a project live in one place per project.** The tier today,
   and later fields (status, owner, URLs, review cadence), are read by every
   process from one file, not from prose tables and lists in five places.

Not in scope: writing the missions themselves in this PR (each is its own
issue and PR, so Jim reviews one product at a time), and platform-tier
projects (`software-development-platform` and `technical-platform`). They
can opt in later by changing the check's tier list, which is one line.

## Question 1: how project metadata is stored today

There is no single place. On 2026-11-28 the facts about a project are
spread over these, each written its own way:

| Fact | Where it lives today | Read by |
| --- | --- | --- |
| Is it a project at all | A folder with a `mise.toml`, listed in `config_roots` in the root `mise.toml` | mise, `//:audit`, `chores user-docs` |
| **Tier** (`software-development-platform`, `technical-platform`, `product`) | **Only a prose table in [decision 0019](../decisions/0019-review-roles.md#tiers).** `standards/reviewers.toml`, which 0019 says will hold it, is not built yet ([review roles phase 1](../../../issues/review-roles-phase-1-model-file-tiers.md)) | Nothing yet |
| On hold | A `# On Hold Projects` bullet list in [issues/README.md](../../../issues/README.md) | Agents reading prose, `standards/user-docs.toml` copies it as `"on hold"` |
| What it is | `README.md` prose, `Cargo.toml` `description` | People |
| Money and usage | `<project>/docs/cost.toml` | `cost check`, pane |
| Threats | `<project>/docs/threat.toml` | `threat check` |
| Requirements | `<project>/docs/requirements.md` (some projects) | Evals, `chores badges` |
| User guide state | `<project>/docs/user/README.md` and the baseline `standards/user-docs.toml` | `chores user-docs` |
| Evals | `<project>/evals.toml` (some projects) | The eval engine |
| Deployed resources | `standards/resources.toml` | The resource audit |
| Technologies used | Exemplar lists in `standards/radar/radar.json` | The tech radar |
| Last deep review | The rotation table in `docs/reviews/README.md` | `deep-project-review` |

So the tier, the one field Jim asked about, is not machine-readable
anywhere yet.

## Proposal: `<project>/docs/project.toml`

One small TOML file per top-level project, next to the `cost.toml` and
`threat.toml` every project already has. It holds the facts that describe
the project and that more than one process reads. Phase 1 starts with two
fields:

```toml
# cookbook: project metadata. Format and rules: "Project metadata" in
# standards/docs/documentation.md (decision 0022).
tier = "product"          # software-development-platform | technical-platform | product
status = "active"         # active | on-hold
```

Later fields join the same file as a process needs them, each added with
its check in the same PR (the audit rejects unknown keys, so a typo can't
hide). Likely next ones: `summary` (one line for pane and the README
index), `owner` (the product-reviewer, once 0019 grants per project),
`urls` (the production host evals and pane link to), and `review_every`
(the deep review cadence).

Rules:

- **Every top-level project has one** (PRJ-001); the keys and values are
  from a closed set (PRJ-002). Sub-projects (`cookbook/web`) inherit their
  parent's file.
- **One reader.** A `projects` module in `chores` parses the files, and
  every other check (`user-docs`, the new `product-docs`, the review roles
  check, `next-task`, pane's snapshot, the deep review rotation) asks it
  instead of keeping its own list.
- **The on-hold list moves here.** `status = "on-hold"` replaces the bullet
  list in `issues/README.md`, and `next-task` reads it.
- **Changing a tier is a strict change.** A PR that adds a `project.toml`
  or changes its `tier` counts as touching the
  `software-development-platform` tier in the review roles check, whatever
  folder the file is in, so moving a project to a laxer tier needs the
  strictest review. `standards/reviewers.toml` keeps only the grants (who
  holds which role over which scope), not the tier list.

Why one file per project and not one central file: it follows
[decision 0006](../decisions/0006-one-file-per-issue.md) (parallel PRs that
each touch a different project never conflict), the file sits with the
project it describes like `cost.toml` and `threat.toml`, and a new project
adds its own file instead of editing a shared one. The central table Jim
may still want to read is generated, not committed: `chores projects`
prints it, and pane shows it.

## Question 2: the product documents

Two files in every `product` project, so the slow-changing part and the
fast-changing part review separately:

### `<project>/docs/mission.md`

- `## Mission`: what the product does, for whom, and why, in a few
  sentences. It changes rarely.
- `## Vision`: what the world looks like for its users if it succeeds,
  about two years out.

### `<project>/docs/roadmap.md`

- `## Product roadmap`: what users will get, as `### Now`, `### Next` and
  `### Later`.
- `## Technical roadmap`: what has to change underneath (architecture,
  platform moves, debt, evals), with the same three subsections.

Each roadmap item is one line that links the issue file or epic that does
the work, or says `(no issue yet)` for a `Later` idea. `Now` items must
link an issue. Roadmaps say *why and in what order*; issue files stay the
unit of work, so nothing is tracked twice.

How the documents relate to what exists: `docs/requirements.md` says what
must stay true and is checked by evals; the mission says why the product
exists, so each requirement should serve it. The user guide says how to
use what already ships; the roadmap says what comes next.

### The check: `chores product-docs`

A Rust command run by `mise run //:audit`, beside `chores user-docs`, for
every project whose `project.toml` says `tier = "product"`:

| Rule | Fails when |
| --- | --- |
| DOC-009 | `docs/mission.md` or `docs/roadmap.md` is missing |
| DOC-010 | A required `##` or `###` section is missing or empty (`Later` may be empty) |
| DOC-011 | A roadmap link points at an issue file or doc that doesn't exist, so closing an issue means updating the roadmap |
| DOC-012 | The baseline is stale: a listed project passes, its issue file is gone, or it is not a product project |

The baseline `standards/product-docs.toml` lists the product projects
without passing documents, each with its issue file, or `"on hold"`. It
only shrinks, exactly like `standards/user-docs.toml`. A new product
project ships with both files.

### Who writes and approves them

- An agent drafts each project's two files from what already exists: the
  README, `docs/requirements.md`, the user guide, its latest deep review
  deck in `docs/reviews/`, its open issue files and its plans. Every claim
  it can't source is marked as a guess for Jim.
- **The mission and vision are Jim's call.** A change to `docs/mission.md`
  counts as the `requirements` domain in the review roles check (like
  `docs/requirements.md`), so it never auto-merges and needs the
  `requirements-reviewer` (Jim today). Roadmap changes are ordinary product
  changes, so agents keep them current as they work issues without waiting
  on Jim.
- The `start-a-new-task` skill gains one step: when the issue it closes is
  linked from a roadmap, update that line in the same PR (DOC-011 enforces
  it).

### Who reads them

- **pane** shows each product's mission and its two `Now` lists on the
  project page.
- **`deep-project-review`** reads the mission, vision and roadmaps first,
  and judges the project against them ("is it heading where it says?").
- **`code-review` and `start-a-new-task`** may later prefer issues a
  roadmap lists under `Now` (a later change to `next-task`'s order, not in
  this plan's phases).

## Phases

Each phase is one issue file, one PR.

1. **Project metadata** ([issue](../../../issues/product-strategy-phase-1-project-metadata.md)).
   Add `docs/project.toml` to all 27 top-level projects with `tier` and
   `status` (tiers from decision 0019, on hold from `issues/README.md`), the
   `chores projects` module and command, PRJ-001 and PRJ-002 in the audit,
   "Project metadata" in `standards/docs/documentation.md`, and move the
   on-hold list. Make `chores user-docs` read `status` instead of its own
   `"on hold"` copies. Update review roles phase 1 to read tiers from here.
2. **Product docs check** ([issue](../../../issues/product-strategy-phase-2-product-docs-check.md)).
   `chores product-docs` (DOC-009 to DOC-012), the baseline, templates in
   `standards/docs/templates/`, "Product docs" in
   `standards/docs/documentation.md`, the AGENTS.md rule, and a
   `write-mission-and-roadmap.md` issue (epic "Product strategy") in each
   active product project.
3. **Write them** (the per-project issues phase 2 adds). One PR per
   product, removing its baseline line. Priority order follows the
   projects Jim has ranked: sparkhill (Blue Eel), cookbook, bear_down,
   cahokia, homer, controlled_chaos, then blueeel, bogan, my_lang,
   psy_ents. luggage, starcraft and profiles stay `"on hold"` with no issue.
4. **Readers** ([issue](../../../issues/product-strategy-phase-4-readers.md)).
   pane's project page, the `deep-project-review` and `start-a-new-task`
   skill steps, and the review roles check treating `docs/mission.md` as
   `requirements` (with review roles phase 2 if that lands first).

## Cost

- Money: $0. Everything is files in the repo and Rust checks in the
  existing audit; no new vendor or service, nothing near a free-tier limit.
  The audit grows by two file reads per project.
- Claude Code effort, against the $25 a month budget: phases 1, 2 and 4
  are one small agent PR each. Phase 3 is ten short documents, one PR each,
  mostly reading existing docs. Altogether about the size of the user
  guides epic.
- Jim's time: reading ten missions and visions once, then only when one
  changes.

## Risks

- **A template answer.** A check proves the sections exist, not that they
  say anything. Mitigation: Jim reviews every mission (the `requirements`
  rule above), and the deep review judges the roadmap against the code.
- **Roadmap drift.** DOC-011 catches links to closed issues, not a `Now`
  list that is stale but still linked. The deep review checks that.
- **Tier downgrade by a project PR.** Covered by treating any `tier`
  change as a `software-development-platform` change.

## Decisions

1. **Where project metadata lives.** Jim chose `docs/project.toml` per
   project on a decision card, 2026-11-28. Options were, all $0 and about one agent PR to build:
   - **`<project>/docs/project.toml` per project (recommended)**: no
     conflicts between parallel PRs, sits with `cost.toml` and
     `threat.toml`, any tier change still gets strict review.
   - One central `standards/projects.toml`: every project on one screen,
     and any edit to it is automatically a strict change, but every new
     project or field edits one shared file.
   - Keep tiers in `standards/reviewers.toml` as 0019 says, and decide
     other fields later: no change now, but metadata stays scattered.

The rest of the plan (two files, the sections, the check, Jim approving
missions) is the recommended default; Jim can change any of it in review.
