# 0026. Every project is independent; anything live it shares has a migration guide in the project

- Status: Accepted
- Decided by: Jim (project thread, 2026-11-28)
- Date: 2026-11-28
- Asked by: Jim
- Standard: [project-isolation.md](../project-isolation.md)
- Builds on: [0017](0017-user-docs.md) (a required doc per project),
  [0021](0021-tenant-scoped-product-data.md) (shared D1 with a move-out
  path) and [0022](0022-project-metadata-and-product-docs.md) ("No
  centralized directories")

## Context

Jim (2026-11-28): "Let's ensure that we include in our standards that each
project is an independent project and should not share live
infrastructure and resources as much as possible and whenever something is
shared, a migration guide should be devised and included in the project.
We want to have every project be independent and isolated (in theory we
could just sell it off or move it to a different repo and it kinda still
works), but in practice we know sharing stuff reduces costs and complexity
so we just want this to be very explicit in each projects design."

Today the projects share a handful of live resources, and nothing in a
project says so. Found on 2026-11-28 by scanning each project's files:

| Shared resource | Used by (besides its owner) |
| --- | --- |
| The one Cloudflare account and deploy token (owner `infra`) | bear_down, cahokia, controlled_chaos, cookbook, design_system, pane, sparkhill, standards (its shared deploy tasks), starcraft, telemetry, tofu_state |
| The one Auth0 tenant (owner `infra`) | cahokia, cookbook |
| The `jimhillr-telemetry` Worker (owner `telemetry`) | bear_down, controlled_chaos |
| The one PostHog project and its free quota | cookbook (pane, cahokia and bear_down are planned) |
| The `jimhillr-flags` Worker and D1 (0018) | none yet |
| The `jimhillr-products` D1 (0021) | none yet |

Every project is also coupled to this repository: shared mise tasks,
workflows, path dependencies and badges on the `evals-results` branch.

## Decision

1. **Every project is independent.** It owns its live resources; sharing a
   live resource with another project is the exception, chosen only when it
   clearly saves money or complexity. Libraries and patterns stay shared.
2. **Sharing is written down in the project.** Each top-level project has
   `docs/isolation.md` with `## Moving out` (what a copy needs outside this
   repo) and `## Shared resources` (`None.`, or one section per shared
   resource with `#### Why it is shared` and `#### Migration guide`).
3. **The known shared resources are one table** in the standard, kept in
   step with the check's list in `chores/src/isolation.rs`. It lists
   platform resources, not projects: which projects use one is found in
   each project's own files, so there is no central directory of projects
   or exemptions.
4. **`chores isolation` checks it** (ISO-001 to ISO-005) in the audit. It
   warns until the epic "Project isolation" has written every guide (one
   issue file per project), then the audit runs it with `--strict`.
5. **A PR that adds sharing updates the guide** in the same PR and names it
   under `## Risk changes`.

## Options considered for the check

| Option | For | Against |
| --- | --- | --- |
| Scan each project's files for known resources, guide in its own docs (chosen) | Finds sharing nobody declared; nothing central about projects; works today | The resource list is central (one table and one Rust constant) |
| A `[[shares]]` table in each project's `docs/project.toml` (0022) | Machine-readable; one metadata file | Declared sharing only, so an undeclared use is missed; `project.toml` is not on `main` yet (PR 922). It can be added later as a cross-check |
| Each owner declares its resource's detection patterns in its own files | Fully per project | Vendor accounts (Cloudflare, Auth0, PostHog) have no owning project; more formats to parse |
| A baseline file of projects allowed to lack a guide | Fails at once on new gaps | A central directory, which Jim ruled out in 0022 |

## Consequences

- Cost: $0. No vendor, no service, no free-tier use. One PR for the
  standard and check, then one short agent PR per project for its guide
  (about 18 small guides) against the Claude Code budget.
- Sharing stays allowed; it just stops being implicit. The guides make a
  later split (a sale, a new owner, a free-tier limit) a known list of
  steps rather than an investigation.
- When `docs/project.toml` lands (0022), a follow-up can add a `shares`
  list there and compare it with the scan.
