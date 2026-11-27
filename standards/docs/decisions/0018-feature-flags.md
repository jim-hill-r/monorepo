# 0018. Feature flags: our own small service on Cloudflare (D1 and a Rust Worker), with per-user modes set from pane

- Status: Proposed
- Decided by: Jim (pending)
- Date: 2026-11-27
- Asked by: Jim
- Builds on: [0004](0004-database-engine.md) (D1 for data a deployed
  service writes), [0007](0007-infrastructure-as-code.md) (IaC),
  [0010](0010-credential-second-approval.md) (credential changes need a
  second approval), [0014](0014-faster-pr-approvals.md) (pane as Jim's
  inbox)

## Context

Jim (2026-11-27):

> I need to do tech selection for feature flags. One of my major initial
> needs is the ability for an admin user to change what modes an app is
> running for a particular logged in user.

So the first job is **per-user modes**: an admin picks a signed-in user
(an Auth0 identity, the `sub` claim `auth_sdk` already reads) and sets which
modes an app runs in for that person, for example a beta UI, a debug
overlay or a new editor in cahokia. Percentage rollouts, A/B splits and
kill switches are useful later, but not the first need.

What our apps look like today:

- **Dioxus web apps on Cloudflare Pages** (cahokia, pane, cookbook,
  bear_down). Sign-in runs in the browser through `auth_sdk` (Auth0,
  authorization code with PKCE). There is no server code behind them yet.
- **Rust Workers** built with `workers-rs` (`telemetry`, `tofu_state`, and
  pane's planned Phase 3 Worker). They compile to WebAssembly, with no Tokio
  runtime.
- **Blue Eel** (sparkhill) stays free of analytics and third-party scripts
  (SPRK-006).
- The repo is **public**, so nothing that names a user (an Auth0 `sub` or
  an email) can be committed.

The principles each option is judged on, from AGENTS.md: cost first and
free tiers only; fewest vendors (GitHub, Cloudflare, Squarespace approved);
open source preferred; automatable and IaC-able, with no clicking in
dashboards; no secrets in git or OpenTofu state; and good Rust support,
since every app is Rust.

## Options

Money is the monthly cost. Free-tier headroom uses
[free-tiers.md](../free-tiers.md). Claude build effort is a rough count of
agent PRs, which spends the $25 a month Claude Code budget. Every option
needs the same two pieces for per-user modes to be safe (see "Threat
model"): a server that checks the Auth0 token before it answers, and an
audit log we control. Those are counted in each option's effort.

| | Option | Money | Free-tier headroom | Claude effort | Open source | IaC and automation | Rust and Workers fit | Admin UI and audit |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| **A** | **Cloudflare Flagship** (public beta since 2026-05-26) | $0 in beta. Pricing after GA is not announced ("we'll share more details on pricing as we approach general availability"). | Limits are generous (5,000 flags an app) and not yet tied to a plan | About 5 PRs | SDKs only; the service is closed | REST API and `wrangler flagship`; no OpenTofu resource found | The Rust SDK needs Tokio, so it can't run in a `workers-rs` Worker, and `workers-rs` 0.9 has no Flagship binding. A Rust Worker would call the evaluation API over HTTP with a token (a Worker secret) or through JS interop. | Cloudflare dashboard (not pane), and a per-flag changelog with field diffs that records the API token, not which admin acted, so we'd still keep our own log |
| B | **OpenFeature with flagd** (CNCF, Apache 2.0) | $0 for the software | flagd is a long-running Go server. No approved vendor runs one free (Workers can't, Cloudflare Containers are paid). | About 6 PRs | Yes | Flags are a JSON file, so they're IaC by nature | A Rust OpenFeature SDK and flagd provider exist, but their in-process mode wants gRPC sync or a file system, neither of which fits a WebAssembly Worker | None. A per-user change is an edit to the flag file and a redeploy, and the file can't name users in a public repo. |
| C | **Unleash or GrowthBook** | Self-hosted: $0 software, but each needs a server and Postgres or MongoDB, with no free host among approved vendors. GrowthBook Cloud has a free plan; Unleash's hosted plans are paid. | Hosted free plans cap seats and features | About 4 PRs, plus Jim's sign-off for a new vendor | Yes (open core) | Terraform providers exist; otherwise their APIs | No Rust server SDK in Workers; we'd call their HTTP APIs | Both have a full admin UI and an audit log, in their own app, not pane |
| D | **PostHog feature flags** | $0 within PostHog's free flag quota (1 million requests a month) | One more use of PostHog's free plan | About 4 PRs | Yes (MIT core) | REST API | HTTP API from Rust; the browser SDK is JavaScript | PostHog's UI and activity log. **Can't be used for Blue Eel** (analytics-free), and widens PostHog's approval beyond session replay, which needs Jim's sign-off. Sends user ids to PostHog. |
| E | **Auth0 `app_metadata`** plus an Auth0 Action that copies the modes into the token | $0 | Inside Auth0's free 25,000 MAU | About 3 PRs | No | The Action is IaC (`infra` auth0 stack); per-user values are runtime data | Works everywhere, since modes ride in the token `auth_sdk` already verifies | Writing a user's metadata needs a Management API token with `update:users`, which can change **any** user's profile, so every admin change is a credential-class power. Modes only change at the user's next sign-in or token refresh. Auth0's free log retention is short, so we'd keep our own log. |
| **F** | **Our own small service**: flag definitions in the repo, per-user overrides and an append-only audit log in D1, a Rust Worker that evaluates, and a pane page for admins | $0 | 1 Worker (of a budget of 25), 1 D1 database (of 10), a few hundred D1 rows a day (of 100,000 written, 5,000,000 read), 1 Access application (pane's, already planned) | About 5 PRs | Yes, ours | Definitions are files reviewed in PRs; the Worker and D1 database are IaC; overrides go through an API agents and scripts can call | Pure Rust, `workers-rs` with the `d1` feature, the same shape as `tofu_state` | A pane page, with our own audit log recording which admin did what |

### Why not A to E

- **A, Flagship**, is the closest fit on vendors (it is Cloudflare), but its
  price after the beta is unknown, which "free tiers only" can't accept for
  something every app would depend on. It also doesn't run in our Rust
  Workers without a token or JS glue, and per-user modes become a list of
  `sub` values inside each flag's rules, edited by a `PUT` that replaces the
  whole flag, so two admin changes at once can overwrite each other. We'd
  still build our own audit log and admin page. Worth another look at GA
  (see "Revisit").
- **B, flagd**, is the most open, but has nowhere free to run and no admin
  path at all for per-user changes.
- **C, Unleash or GrowthBook**, is a new vendor or a server to host, for
  features (experiments, analytics) we don't need yet.
- **D, PostHog**, can't serve Blue Eel, and turns an analytics vendor into a
  dependency of every app's behaviour.
- **E, Auth0 metadata**, is the least code, but its write token can edit
  every user, and changes wait for the next sign-in. It is the right home
  for **roles** (who is an admin), not for modes.

## Decision

Option F. Each item is a default, so Jim can answer "yes to all".

1. **Flag definitions live in the repo.** Each project that uses flags has
   `docs/flags.toml`: every flag's key, type (boolean, string or a named
   set of modes), default, description, owner and an optional expiry date.
   Defining, renaming or deleting a flag is a reviewed PR, so definitions
   are as IaC as the rest of the repo. A CI check fails a flag past its
   expiry, so flags don't pile up. Definitions never name a user.
2. **Per-user overrides live in D1**, because they are runtime data a
   deployed service writes and the public repo can't hold user ids
   (decision 0004). One database, `jimhillr-flags`, with two tables:
   `overrides (app, flag, sub, value, set_by, set_at, expires_at)` and
   `audit (id, at, actor, app, flag, sub, before, after, reason,
   request_id)`. The database is created by the `infra` cloudflare stack.
3. **One Rust Worker, `jimhillr-flags`**, built with `workers-rs` and the
   `d1` feature, like `tofu_state`:
   - `GET /v1/apps/{app}/flags` returns every flag's value for the caller.
     It requires an Auth0 **access token** (`Authorization: Bearer`) and
     verifies its signature against Auth0's JWKS, its issuer, audience and
     expiry, then takes `sub` **only** from the verified token. Without a
     valid token it returns the defaults. CORS allows only each app's own
     origins.
   - `/admin/*` lists users with overrides, sets and clears an override,
     and reads the audit log. It sits behind a **Cloudflare Access**
     application allowing only Jim's email, and the Worker also verifies the
     `Cf-Access-Jwt-Assertion` header on every call, so a misconfigured
     Access policy fails closed (the same pattern as pane Phase 3). The
     actor recorded in the audit log is the email from that verified
     Access token, never a value the browser sends.
   - Every write to `overrides` and its `audit` row happen in one D1 batch,
     so there is no change without a log entry. `audit` is append-only: the
     Worker never issues `UPDATE` or `DELETE` on it, and SQLite triggers
     reject them.
4. **A shared Rust crate, `flags`**, gives apps one small, OpenFeature-shaped
   API (`bool_value`, `string_value`, `mode` with a default) over the
   Worker, for Dioxus web apps and Workers alike. It caches the answer for
   the session and falls back to the definition's default when the Worker
   can't be reached, so a flags outage never breaks an app. Keeping the
   OpenFeature shape means a later move to Flagship (or any OpenFeature
   provider) changes the crate, not every app.
5. **`auth_sdk` asks for an access token for the flags API.** That needs an
   Auth0 API (`auth0_resource_server`) with the audience
   `https://flags.jimhillr` and each app's client allowed to request it.
   Under decision 0010 that is a credential change, so it applies only
   after Jim's second approval in `infra-credentials`. No new secret is
   needed: verifying a token needs only Auth0's public keys.
6. **The admin UI is a pane page, "Flags".** It lists each app's flags from
   `docs/flags.toml` (pane's snapshot already reads the repo), lets Jim
   search for a user, set or clear a mode with a required reason and an
   optional expiry, and shows the audit log. Until pane Phase 3 moves pane
   behind Access, the page calls the flags Worker's `/admin/*` routes,
   which have their own Access application.
7. **Flags never grant authorization.** A mode changes how an app looks or
   behaves for a person; it is never the thing that lets someone read or
   change data they otherwise couldn't. Who is an admin stays an Auth0
   role, which is a credential change under decision 0010. Anything
   privileged that a mode turns on is also checked on the server. A CI
   check refuses flag keys or modes named like permissions (`admin`,
   `role`, `superuser`, `bypass`).
8. **Jim sees every change.** The weekly human issues digest lists the
   week's audit entries (count by app and flag, and any override with no
   expiry), and pane's Today page shows changes from the last day.

## Threat model

Setting someone's modes is a privileged action, so the flags Worker gets
its own `docs/threat.toml` when it's built. The main threats:

| STRIDE | Threat | Mitigation |
| --- | --- | --- |
| Spoofing | A user asks for another user's modes, or claims to be an admin | `sub` comes only from a verified Auth0 access token; admin calls need a verified Access token for Jim's email (item 3) |
| Spoofing | A stolen or forged token | Signature, issuer, audience and expiry are all checked; access tokens are short-lived; the audience is specific to the flags API |
| Tampering | Someone changes another user's overrides | Only `/admin/*` writes, behind Access and the Worker's own check; overrides are keyed by the verified `sub` |
| Tampering | An agent ships Worker or pane code that writes overrides or weakens the check | Same exposure as pane in decision 0010: agents can merge code. Contained by item 7 (a mode can't grant access), the append-only audit and Jim's weekly digest of changes (item 8), and code review on every PR touching `flags/` or the pane Flags page (a protected path for the reviewer account) |
| Repudiation | An admin denies making a change | Every change has an audit row in the same batch, with the actor from the verified Access token, before and after, a reason and a request id; the table rejects updates and deletes |
| Information disclosure | Leaking who has which modes, or user ids | A user only ever gets their own values; listing users is admin-only; no user ids in the repo, logs or snapshot; overrides and audit stay in D1 |
| Denial of service | Flooding the Worker uses up the free 100,000 requests a day | Apps cache per session; the Worker rate-limits per `sub` and per IP; on any failure apps fall back to defaults (item 4), so the worst case is everyone gets defaults |
| Elevation of privilege | A mode is used as a back door to admin features | Item 7: flags never grant authorization, modes that look like permissions fail CI, and privileged features are checked on the server |

What it doesn't protect against: Jim's own Auth0 or Cloudflare Access
session being taken over (use a passkey), and someone with D1 write access
through the Cloudflare deploy token editing the tables directly. The deploy
token already has D1 Write (`jimhillr-deploy`); the weekly digest compares
the audit log with the overrides table and reports any override with no
matching audit row.

## Phases

1. **The `flags` Worker, D1 schema and the `flags` crate**, with tests for
   token verification, the batch write and the append-only triggers, and
   its threat model and `docs/cost.toml`. About 2 PRs.
2. **IaC**: the D1 database and Worker in the cloudflare stack, the Auth0
   API and client grants in the auth0 stack (credential change, Jim's
   second approval), and the Access application, with their
   `standards/settings.toml` entries. About 1 PR.
3. **The pane Flags page** and the digest lines. About 1 PR.
4. **First app: cahokia**, with one real mode, its `docs/flags.toml`, an
   eval that checks the default and an override end to end. About 1 PR.

## Cost

$0 a month. One Worker (budget 25), one D1 database (of 10) with a few
hundred rows written a day at most, one Access application inside the
free 50 users, one Auth0 API on the free plan. No new vendor and no new
secret. Claude build effort is about 5 PRs.

## Revisit

Look at Cloudflare Flagship again when it reaches general availability, if
its free plan covers our usage and `workers-rs` gains a binding. The
`flags` crate's OpenFeature shape keeps that a change in one crate.
Percentage rollouts and A/B splits, if we need them, are added to the
Worker first and are the other trigger to revisit.

## Consequences

- Every new flag is a PR to a project's `docs/flags.toml`; every per-user
  change is a logged action in pane.
- The tech radar lists Flagship and OpenFeature in Assess, and flagd,
  Unleash and GrowthBook in Hold, with this record as the source.
- Apps gain one network call per session, cached, with defaults on failure.
