# 0021. Product runtime data in one shared D1 database, in tenant-scoped tables

- Status: Accepted
- Decided by: Jim (approved and merged PR 915, 2026-11-28)
- Date: 2026-11-28
- Asked by: Jim (project thread, 2026-11-28)
- Plan: [2026-11-user-feedback.md](../plans/2026-11-user-feedback.md#data-where-product-data-lives)
- Builds on: [0004](0004-database-engine.md) (files first, then committed
  SQLite, D1 for data a deployed service writes),
  [0018](0018-feature-flags.md) (management apps), 0019 (tiers)

## Context

User feedback and support (decision 0020) are the first product data that
a deployed service must write at runtime and that must not be public. Jim,
2026-11-28: "This opens up a big questions about data warehousing and
ensuring different products are generic and don't use shared database
resources without exploding our cost. I think shared DB infra with
tenatized tables is okay as it would be easy to migrate to isolated
tables if necessary." And: "Assume products are owned by different
companies during architecture."

So we need a rule for every product's runtime data that keeps products
separate, stays on free tiers, and lets one product move to its own
storage without rework.

The limits that shape it (Workers Free, checked 2026-11-28): D1 allows
10 databases per account, 500 MB each and 5 GB in all, 100,000 rows
written and 5,000,000 read a day; since 2026-09-01 queries fail past the
daily limits until midnight UTC. Two databases are already planned or
used (`tofu_state`, `jimhillr-flags`).

## Options

| Option | Money | Free-tier use | Isolation | Moving a product out | Notes |
| --- | --- | --- | --- | --- | --- |
| Public files on a data branch (today's bear_down votes) | $0 | None | None: public | n/a | Fine for aggregates, not for people's words |
| A private GitHub repo per product | $0 | None | Strong | Already separate | Files can't take runtime writes from a Worker without a token that can write a repo; slow and awkward |
| Analytics Engine | $0 | 100,000 writes a day | By dataset | Re-export | 3-month retention, built for metrics, not records with replies |
| **One shared D1, a `tenant` column in every table's key** | **$0** | **1 of 10 databases; budgets per tenant** | **By library and tests** | **Export one tenant's rows to a new D1** | **Recommended** |
| One shared D1, a table set per tenant (`blue_eel_feedback`) | $0 | 1 of 10 | By table names | Copy the tables | Schema changes repeat per tenant; more migrations to keep in step |
| A D1 database per product | $0 | 1 of 10 per product: runs out at about 8 products | Strong, by binding | Already separate | Best isolation, but the 10-database cap is the cost Jim worried about |
| A Durable Object with SQLite storage per tenant | $0 on the free plan | Counts against Durable Object requests and storage | Strong, by object | Export from the object | Isolated by design, but reporting across tenants needs fan-out code; more to build |
| Basin Catalog (R2 Iceberg) as a warehouse | Free allowance, but needs an R2 subscription with a payment card | n/a | By table | n/a | Ruled out by the free-tier rule (no payment method) |
| Hosted Postgres with row-level security (Neon, Supabase) | $0 on free plans | Small free databases, may pause | Strong, in the database | `pg_dump` per tenant | New vendor; row-level security is the one thing D1 lacks |

## Decision

1. **One shared D1 database, `jimhillr-products`,** created by the `infra`
   cloudflare stack, holds the runtime data of every product. Platform
   data (state, flags) keeps its own databases.
2. **Every table is tenant-scoped.** `tenant TEXT NOT NULL` is the first
   column of the primary key and of every index. A tenant is a product
   (`blue_eel`, `bear_down`), never a person.
3. **Only the `tenant_store` library touches it.** A product's Worker opens
   `TenantStore::open(env)`, which reads the tenant from the Worker's own
   configuration at deploy time, never from a request. Every query in the
   library binds the tenant; a unit test fails any query string without
   the tenant condition, and products write no raw SQL against the shared
   database. D1 has no row-level security, so this is the isolation
   boundary, and it is a requirement (FDBK-011) with tests.
4. **Each tenant has a budget.** Daily rows written and bytes stored per
   tenant live in `standards/radar/limits.toml` (shares of the account's
   100,000 rows a day and 5 GB). `tenant_store` counts its own writes and
   refuses past the budget, so one product can't exhaust another's share,
   and `free-tiers.sh` reports each tenant's use weekly. Budgets are raised
   only by Jim, as for every free-tier budget.
5. **Moving out is designed in.** `chores tenant-export <tenant>` writes
   one tenant's rows as a `.sql` file that builds the same schema in a new
   database. A product moves to its own D1 (or off our platform) when it
   passes 100 MB, needs its own backups or region, or changes owner: a new
   binding and one import, no code change.
6. **Files stay the record for what can be public.** Weekly workflows
   write aggregates (counts, topics, average ratings) to data branches as
   JSON Lines, per decision 0004. Raw text people wrote stays in D1 and is
   deleted on each table's stated retention (12 months for feedback and
   closed support requests).
7. **Warehouse later.** Cross-product reporting reads the aggregate files
   and, if needed, read-only queries over the shared database. A warehouse
   (Basin Catalog or another) gets its own decision when a product needs
   one and the free-tier rule allows it.

## Consequences

- Products stay generic: each reads and writes only its own tenant through
  one library, so the same feedback, support and future account code works
  for every product unchanged.
- One database for all products saves nine of the ten D1 slots, at the
  price of isolation by code rather than by binding. The tests and the
  export path are what make that acceptable.
- A product's Worker holds a binding that could, in principle, read
  another tenant with hand-written SQL. Review and the "no raw SQL" check
  guard it, and the threat model lists it.
- Cost: $0, one D1 database, a few rows a day per product; about one agent
  PR to build `tenant_store` and the database.
