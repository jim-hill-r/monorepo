# Project isolation

Every top-level project is an independent project. In theory we could move
it to another repository, or sell it, and it would still mostly work. So a
project avoids sharing live infrastructure and resources with other
projects. Where it does share one, because sharing saves money or
complexity, the project says so explicitly in its own docs and carries a
migration guide for separating it out
([decision 0026](decisions/0026-project-isolation.md)).

Jim, 2026-11-28: "each project is an independent project and should not
share live infrastructure and resources as much as possible and whenever
something is shared, a migration guide should be devised and included in
the project. We want to have every project be independent and isolated (in
theory we could just sell it off or move it to a different repo and it
kinda still works), but in practice we know sharing stuff reduces costs and
complexity so we just want this to be very explicit in each projects
design."

This builds on "Products are separate companies" in `AGENTS.md`
([decision 0021](decisions/0021-tenant-scoped-product-data.md)): the
technical platform shares crates, patterns and infrastructure definitions,
not live services a product's requests pass through.

## What counts as shared

| Shared | Examples | Rule |
| --- | --- | --- |
| **Live resources** | One Cloudflare account, one Auth0 tenant, a Worker or database another project runs, one vendor project's free quota | Avoid. When it is worth it, document it with a migration guide (below). |
| **Repository coupling** | `standards/tasks/` included from `mise.toml`, workflows in `.github/workflows/`, path dependencies such as `auth_sdk`, badges on the `evals-results` branch | Allowed: it is code and patterns, not live state. The guide's `## Moving out` section says what to copy or replace. |
| **Libraries and patterns** | `auth_sdk`, `design_system`, `chores` | Encouraged. Prefer a versioned dependency a moved project could pin. |

A project's own resources (its own Worker, Pages project, D1 database,
secrets, Auth0 application) are not shared, even when they live in the
shared Cloudflare account or Auth0 tenant: the account or tenant is the
shared part.

## Known shared live resources

The check (below) finds these in each project's files. The project that
runs a resource is its owner and documents who uses it instead.

| Id | What is shared | Owner | Found by | Migration in short |
| --- | --- | --- | --- | --- |
| `cloudflare-account` | The one Cloudflare account (Workers and Pages free plans, the `jim-hill-r.workers.dev` subdomain, account-wide free-tier limits) and the `jimhillr-deploy` token in GitHub secrets | `infra` | a `wrangler.toml`, `wrangler deploy`, or a `standards/tasks/cloudflare-*` or `bevy-game.toml` include | A new free account, its own token and account id as the new repo's secrets, redeploy, then move any custom domain |
| `auth0-tenant` | The one Auth0 tenant (`dev-jdadpn4pckxevrv5.us.auth0.com`), its users and the free plan's limits | `infra` | the tenant's domain | A new tenant, its own application and API, then a user export and import (users re-verify or reset passwords) |
| `telemetry-worker` | The `jimhillr-telemetry` Worker, its `telemetry` Analytics Engine dataset and the weekly export to the `feedback-data` branch | `telemetry` | `jimhillr-telemetry` | Deploy a copy of `telemetry` under a new name and point the client at it, or drop the events |
| `posthog-project` | The one PostHog project, its public `phc_` key and its 5,000 free recordings a month | none (vendor account) | `phc_` | A new PostHog project or account and its own key in the page; old recordings stay behind |
| `flags-service` | The `jimhillr-flags` Worker and D1 database ([0018](decisions/0018-feature-flags.md)), when it is built | none yet | `jimhillr-flags` | Deploy a copy of the flags service and export the project's modes |
| `products-d1` | The `jimhillr-products` D1 database with tenant-scoped tables ([0021](decisions/0021-tenant-scoped-product-data.md)), when it is built | none yet | `jimhillr-products`, `tenant_store` | Export the tenant's rows to a new D1 database and point the binding at it |

A new shared resource is added in one PR: a row here, an entry in
`SHARED` in `chores/src/isolation.rs` (a test checks they agree), and a
section in each user's guide. Before adding one, say in the PR why a
project-owned resource would not do (usually a free-tier limit or a vendor
that allows one account).

## The isolation guide

Every top-level project has `<project>/docs/isolation.md`:

```markdown
# <project>: isolation

## Moving out

What a copy of this folder needs to build, test and deploy from another
repository: the shared mise tasks and workflows to copy, path dependencies
to replace with versions, secrets and variables to set, and badges or data
branches that point back here.

## Shared resources

None.
```

When the project uses a shared resource, `## Shared resources` has one
`###` section per resource, its heading naming the id in backticks:

```markdown
### Cloudflare account (`cloudflare-account`)

#### Why it is shared

One free account holds every project's Workers and Pages; a second account
would split the free limits but add a second set of tokens to rotate.

#### Migration guide

1. Create a free Cloudflare account for the new owner.
2. Make a deploy token with Workers Scripts:Edit and Pages:Edit only.
3. Set `CLOUDFLARE_API_TOKEN` and `CLOUDFLARE_ACCOUNT_ID` in the new repo.
4. Run `mise run deploy`, check the new URL, update links and DNS.
5. Retire the old resources in `standards/resources.toml`.

Cost after separating: $0 (free plan). Verify: the evals pass against the
new URL.
```

A migration guide says the steps, what it costs after separating (money
and free-tier use), what data moves or stays behind, and how to check it
worked. It is written for someone who has the project folder and not the
rest of this repository.

## The check

`chores isolation` (ISO-001 to ISO-005, run by `mise run //:audit`) reads
each project's own files, with no central list of projects or exemptions:

| Rule | Fails when |
| --- | --- |
| ISO-001 | The project has no `docs/isolation.md`. |
| ISO-002 | The guide has no `## Moving out` or `## Shared resources`, or one is empty. |
| ISO-003 | The project's files use a known shared resource the guide has no `###` section for. |
| ISO-004 | A resource's section has no `#### Why it is shared` or `#### Migration guide` with text. |
| ISO-005 | A `###` section names an id that is not a known shared resource. |

The scan reads the same file types as the cost check and skips `docs/`,
`issues/`, `tests/`, fixtures and build output. A file that only names
resources (a list of settings, the check itself) holds the text
`isolation-check: names only` and is skipped.

Findings are warnings while the epic "Project isolation" writes each
project's guide (one issue file per project). When the last guide lands,
the audit switches to `chores isolation --strict`, and a missing or stale
guide fails it.

## When a PR shares something new

A PR that makes a project use a shared resource, or turns a project's own
resource into a shared one, updates that project's guide in the same PR
and says so under `## Risk changes`. Prefer a project-owned resource when
it costs the same, usually $0 on a free tier.
