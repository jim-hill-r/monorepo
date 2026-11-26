# 0004. Files first, committed SQLite next, Neon when a hosted database is needed

- Status: Proposed (needs Jim's sign-off: Neon is not an approved vendor)
- Date: 2026-11-26
- Decided by: Jim (pending)

## Context

No project in the monorepo uses a database today. Jim (2026-11-26):

> Lets try to avoid results in databases for as long as possible. Lets try to
> use artifacts in the monorepo for any dev work whenever possible. Also I
> think I want to use neon as my database engine but we need to do a DB
> evaluation first. Lastly, we can try using locally committed SQL DB files
> whenever we need a proper DB in repo.

The first rule and the committed-SQLite rule are now in
[AGENTS.md](../../../AGENTS.md) ("Files before databases"). This record
covers the rest: which hosted database to use once a deployed service really
needs one, judged against the rules we already have:

- **Free tiers only** ([free-tiers.md](../free-tiers.md)): never pay, and
  prefer services that stop rather than bill.
- **Vendors** (AGENTS.md): cost first, then fewest vendors. Cloudflare is
  approved; Neon, Turso and Supabase each need Jim's sign-off.
- Our code is mostly **Rust**, and our services deploy to **Cloudflare
  Workers and Pages**.

The options compared are Neon (Jim's leaning), Cloudflare D1, Turso and
Supabase, plus the in-repo SQLite baseline. Limits were read on 2026-11-26
from each vendor's pricing and limits pages (sources at the end). They change;
check the source before relying on a number.

## Free tiers

All four can be used with no card on file, and on all four a free account
stops or blocks at a limit instead of billing. None of them costs money as
long as nobody upgrades, so cost does not separate them; limits and fit do.

| | Neon Free | Cloudflare D1 (Workers Free) | Turso Free | Supabase Free |
| --- | --- | --- | --- | --- |
| Engine | Postgres | SQLite | libSQL (SQLite fork) | Postgres |
| Databases | 100 projects, 10 branches each | 10 databases | 100 databases | 2 active projects |
| Storage | 1 GB per project, 20 GB total | 500 MB per database, 5 GB total | 5 GB total | 500 MB per project |
| Compute / queries | 100 CU-hours per project a month (about 400 hours at 0.25 CU); scales to zero after 5 minutes idle | 5M rows read and 100,000 written a day | 500M rows read and 10M written a month | Shared CPU; unlimited API requests |
| Egress | 5 GB per project a month | Included in Workers | 3 GB of embedded syncs a month | 5 GB a month |
| Point-in-time restore | 6 hours | 7 days (Time Travel) | 1 day | None (no backups on Free) |
| Idle behaviour | Scales to zero; never paused | Always on | Always on | **Paused after a week of low activity** |
| Past a limit | Compute suspended until next month, or writes that grow storage fail; "no overages on Free" | Queries error until 00:00 UTC; at the storage limit, writes fail | All databases blocked once any one metric is exceeded | Not documented for Free (treat as a risk) |
| Cheapest paid plan (not to be used) | Launch, pay as you go (about $15 a month for a small DB) | Workers Paid, $5 a month | Developer, $5.99 a month | Pro, $25 a month |

## Rust support

- **Neon** is plain Postgres, so every Rust Postgres crate works: `sqlx`,
  `tokio-postgres`, `diesel`, `sea-orm`. Nothing Neon-specific to learn.
- **D1** has a first-class binding in `workers-rs` (`worker::D1Database`),
  usable only inside a Worker. Outside Workers there is only Cloudflare's
  REST API, which is fine for scripts but not a driver.
- **Turso** has the `libsql` Rust crate (remote, local file and embedded
  replicas), and Turso's own engine is written in Rust. Plain `rusqlite`
  also reads the same files, which matches the committed-SQLite rule.
- **Supabase** is Postgres, so the same crates as Neon work for the
  database. Its extras (auth, storage, PostgREST) are HTTP APIs with only
  community Rust clients.

## Cloudflare Workers compatibility

- **D1** is native: a binding in `wrangler.toml`, no network hop, no secret
  to manage, and `wrangler dev` runs it locally against a SQLite file.
- **Neon** works from Workers through
  [Hyperdrive](https://developers.cloudflare.com/hyperdrive/), which is on
  the Workers Free plan since April 2025 and pools connections close to the
  database. From Rust Workers that means `tokio-postgres` over a
  `worker::Socket` (the `workers-rs` Hyperdrive example). Neon's own
  serverless driver is JavaScript only. Hyperdrive has its own daily query
  limit on Free, which would need a row in free-tiers.md.
- **Turso** works from Workers over HTTP (the `libsql` crate's remote mode,
  or its HTTP API); there is no binding, so a token is a Worker secret.
- **Supabase** works from Workers over HTTP (PostgREST) or through
  Hyperdrive like Neon.

## Branching and dev workflow

- **Neon** is the strongest here. Branches are copy-on-write copies of the
  whole database, made in about a second, and Neon publishes GitHub Actions
  to create one per pull request and delete it on close. That fits our
  preview-build flow (decision 0001). The Free limit of 10 branches per
  project means a cleanup step is required and preview branches must go
  through `free-tiers.sh` like Pages projects.
- **D1** has no branches. A preview needs a whole new database (10 per
  account on Free), so per-PR databases are not practical. Local dev with
  `wrangler dev` is very good.
- **Turso** can create a database from another (`--from-db`), which works as
  a branch, and has 100 databases to spend. Local dev can use a plain SQLite
  file.
- **Supabase** branching needs a paid plan. Local dev runs the whole stack
  in Docker through the Supabase CLI, which is heavy for our cloud sandboxes.

## Lock-in and the path from committed SQLite

- **Neon**: lowest lock-in of the hosted options. It is standard Postgres;
  `pg_dump` moves it anywhere. The cost is a dialect change: data that starts
  as a committed SQLite file has to be converted to Postgres when it moves to
  Neon (types, `AUTOINCREMENT`, date functions). Writing queries in `sqlx`
  and keeping to plain SQL keeps that small.
- **D1**: SQLite dialect, so a committed `.sql` dump loads straight in with
  `wrangler d1 execute --file`, and exports back out the same way. The
  lock-in is the runtime: D1 is only reachable as a Workers binding.
- **Turso**: SQLite files in and out (`turso db import` takes a local file),
  and the engine is open source. Turso announced on 2026-10-02 that it is
  being acquired by Supabase; it says the service keeps running, but its
  plans could change.
- **Supabase**: the database is portable Postgres, but using its auth,
  storage or edge functions ties code to Supabase. We already use Auth0 for
  sign-in, so most of its value would go unused. Projects that pause after a
  week idle are a bad fit for low-traffic hobby services.

## Decision

1. **Files first.** Data and results are committed files, as AGENTS.md says.
   This is the default for every project and covers the eval engine (its
   plan is being changed to drop D1 for committed files).
2. **Committed SQLite next.** When a project needs real queries over data
   in the repo, it commits a SQLite `.sql` dump (or a small `.sqlite` file)
   and reads it with `rusqlite` (feature `bundled`). No account, no vendor,
   nothing to run out of.
3. **Neon for the first hosted database**, when a deployed service must
   write data at runtime, with these conditions:
   - Neon becomes an approved vendor exception once Jim accepts this record.
   - Free plan only, no card. Add a Neon section to
     [free-tiers.md](../free-tiers.md) (100 CU-hours and 5 GB egress per
     project a month, 1 GB storage, 10 branches) and a Hyperdrive row, and
     extend `free-tiers.sh` so preview branches are created and deleted only
     through the guard.
   - Use one Neon project per service, a branch per pull request deleted
     when the PR closes, and Hyperdrive to reach it from Workers.
   - Write migrations and queries in plain SQL with `sqlx`, so the database
     can move with a `pg_dump`.

### Why Neon and not D1

The vendor rule says that when cost is equal, fewer vendors wins, and D1 is
already approved. D1 is the right answer if the only need is a small table
behind one Worker. Neon is still recommended because:

- per-PR branches are the dev workflow we want, and D1 has nothing like it;
- standard Postgres has the least lock-in and works from any Rust code, not
  only inside a Worker;
- monthly CU-hours with scale to zero suit bursty hobby traffic better than
  D1's daily caps, and nothing is paused when idle.

If Jim prefers to keep the vendor count down, D1 is the fallback: same
SQLite dialect as the committed files, no new vendor, but no branching.

Turso and Supabase are not recommended: Turso's whole-account block on any
one limit and its pending acquisition make it a riskier free tier, and
Supabase's two projects, weekly pausing and paid-only branching fit worst.

## Consequences

- No database is created now. The next PR that needs a hosted database
  adds the Neon free-tier section, the guard and its first project.
- Jim signs off by merging this PR (labelled `needs-human`). The same PR
  lists Neon under the approved exceptions in AGENTS.md, so merging it
  approves the vendor.
- Moving from committed SQLite to Neon needs a SQLite-to-Postgres
  conversion; keeping SQL plain keeps it cheap.

## Sources

Read 2026-11-26.

- Neon: [plans and limits](https://neon.com/docs/introduction/limits),
  [pricing](https://neon.com/pricing),
  [free plan FAQ](https://neon.com/faqs/free-plan-limits-and-quotas).
- Cloudflare: [D1 limits](https://developers.cloudflare.com/d1/platform/limits/),
  [D1 FAQ](https://developers.cloudflare.com/d1/reference/faq),
  [Hyperdrive on the Free plan](https://developers.cloudflare.com/changelog/2025-04-08-hyperdrive-free-plan/),
  [Hyperdrive pricing](https://developers.cloudflare.com/hyperdrive/platform/pricing).
- Turso: [pricing](https://turso.tech/pricing),
  [Turso is joining Supabase](https://turso.tech/blog/turso-is-joining-supabase).
- Supabase: [pricing](https://supabase.com/pricing),
  [free project pausing](https://supabase.com/kb/faqs/platform/free-project-pausing),
  [branching](https://supabase.com/docs/guides/deployment/branching/overview).
