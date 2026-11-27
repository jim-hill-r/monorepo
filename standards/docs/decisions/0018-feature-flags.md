# 0018. Feature flags: our own flags service on Cloudflare, with a console where app managers set modes per user

- Status: Proposed
- Decided by: Jim (pending)
- Date: 2026-11-27
- Asked by: Jim
- Builds on: [0004](0004-database-engine.md) (D1 for data a deployed
  service writes), [0007](0007-infrastructure-as-code.md) (IaC),
  [0010](0010-credential-second-approval.md) (credential changes need a
  second approval)

## Context

Jim (2026-11-27):

> I need to do tech selection for feature flags. One of my major initial
> needs is the ability for an admin user to change what modes an app is
> running for a particular logged in user.

and, on the first draft of this record:

> So any solution that requires PRs to change a flag is not allowed. In
> this context an admin is an application admin not me as the dev admin.
> Lets consider better nomenclature for an application admin that is in
> charge of app behavior but is not a software engineer. So the solution
> needs to provide this layperson admin the ability to alter behavior on
> an app by user without touching git.

So the first job is **per-user modes, set by a non-engineer**: a person
in charge of an app picks one of its signed-in users (an Auth0 identity,
the `sub` claim `auth_sdk` already reads) and changes which modes the app
runs in for them, for example a beta editor, a simpler layout or a
debug overlay in cahokia. They do it from a web page, with no GitHub
account, no git, no PR and no Cloudflare or Auth0 dashboard. Percentage
rollouts, A/B splits and kill switches are useful later, but not the first
need.

### What to call this person

They are in charge of how an app behaves for its users, but don't write
code, and "admin" is easily confused with Jim's own developer and
infrastructure rights. This record calls them an **app manager**, and
uses these words throughout:

| Word | Means |
| --- | --- |
| **App manager** | A non-engineer, signed in with Auth0, who manages one or more apps' modes for their users. Holds the `manager` role for each app they manage. |
| **Mode** | A named way an app can behave (a flag, in engineering terms), with a plain-language name and description, and a fixed set of choices. |
| **Setting a mode** | Choosing one of a mode's choices for one user, or for everyone in the app. |
| **Console** | The web page where app managers set modes and see the history of changes. |
| **Developer** | Jim and the agents: they add modes to an app's code and deploy it, and appoint app managers. |

Other names considered: *operator* (sounds like infrastructure),
*moderator* (sounds like content), *app owner* (sounds like it can change
anything, including who else manages), *support admin* (still "admin").
Jim can pick another; only the role name and labels change.

### What our apps look like

- **Dioxus web apps on Cloudflare Pages** (cahokia, pane, cookbook,
  bear_down). Sign-in runs in the browser through `auth_sdk` (Auth0,
  authorization code with PKCE). There is no server code behind them yet.
- **Rust Workers** built with `workers-rs` (`telemetry`, `tofu_state`).
  They compile to WebAssembly, with no Tokio runtime.
- **Blue Eel** (sparkhill) stays free of analytics and third-party scripts
  (SPRK-006).
- The repo is **public**, so nothing that names a user (an Auth0 `sub` or
  an email) can be committed.

The principles each option is judged on, from AGENTS.md: cost first and
free tiers only; fewest vendors (GitHub, Cloudflare, Squarespace
approved); open source preferred; automatable and IaC-able; no secrets in
git or OpenTofu state; good Rust support. And now, from Jim: **an app
manager changes modes without git**, and **without access to any vendor
dashboard**, since a vendor dashboard login gives far more than one app's
modes.

## Options

Money is the monthly cost. Free-tier headroom uses
[free-tiers.md](../free-tiers.md). Claude build effort is a rough count of
agent PRs, which spends the $25 a month Claude Code budget. Every option
needs a server that checks the user's Auth0 token before answering (so
nobody can ask for someone else's modes), and that is counted in each
option's effort.

| | Option | Money | Free-tier headroom | Claude effort | Open source | Rust and Workers fit | Where an app manager works, and the audit trail |
| --- | --- | --- | --- | --- | --- | --- | --- |
| A | **Cloudflare Flagship** (public beta since 2026-05-26) | $0 in beta. Pricing after GA is not announced ("we'll share more details on pricing as we approach general availability"). | Generous limits (5,000 flags an app), not yet tied to a plan | About 6 PRs | SDKs only; the service is closed | The Rust SDK needs Tokio, so it can't run in a `workers-rs` Worker, and `workers-rs` 0.9 has no Flagship binding: a Rust Worker would call the evaluation API over HTTP with a token, or use JS interop | Flagship's UI is the Cloudflare dashboard, and an app manager can't be given a dashboard login. So we'd still build our own console, calling Flagship's API with a write token. Per-user values become lists of `sub` values inside each flag's rules, edited by a `PUT` that replaces the whole flag, so two managers' changes can overwrite each other. Its changelog records the API token, not which manager acted. |
| B | **OpenFeature with flagd** (CNCF, Apache 2.0) | $0 for the software | flagd is a long-running Go server; no approved vendor runs one free (Workers can't, Cloudflare Containers are paid) | About 7 PRs | Yes | The Rust flagd provider's in-process mode wants gRPC sync or a file system, neither of which fits a WebAssembly Worker | flagd reads flag files and has no UI or write API, so per-user changes mean editing files: ruled out by "no git". |
| C | **Unleash or GrowthBook** | Self-hosted: $0 software, but each needs a server and Postgres or MongoDB, with no free host among approved vendors. GrowthBook Cloud has a free plan with a capped number of seats; Unleash's hosted plans are paid. | Seat caps on hosted free plans | About 5 PRs, plus Jim's sign-off for a new vendor | Yes (open core) | No Rust SDK for Workers; we'd call their HTTP APIs | Both have a full UI for non-engineers, with an audit log. But it is a separate sign-in (not our Auth0), each manager takes a seat, and the UI is built for engineers running experiments, not for "change this user's mode". |
| D | **PostHog feature flags** | $0 within PostHog's free flag quota (1 million requests a month) | One more use of PostHog's free plan | About 5 PRs | Yes (MIT core) | HTTP API from Rust | An app manager would need a PostHog login, which also shows session replays and analytics: far more than they should see. **Can't be used for Blue Eel** (analytics-free), and widens PostHog's approval beyond session replay. |
| E | **Auth0 user metadata** plus an Auth0 Action that copies modes into the token | $0 | Inside Auth0's free 25,000 MAU | About 4 PRs | No | Works everywhere, since modes ride in the token `auth_sdk` already verifies | Managers can't use the Auth0 dashboard, so we'd build the console anyway, and its backend would need a Management API token with `update:users`, which can change **any** user's profile in every app. Changes only reach the user at their next sign-in or token refresh. |
| **F** | **Our own small service**: a `jimhillr-flags` Worker in Rust, modes and settings in D1, and a console web app for app managers | $0 | 2 Workers or Pages projects (budgets 25 and 25), 1 D1 database (of 10), a few hundred D1 rows a day (of 100,000 written, 5,000,000 read) | About 6 PRs | Yes, ours | Pure Rust, `workers-rs` with the `d1` feature, the same shape as `tofu_state`; the console is Dioxus, like every other app | The console signs managers in with the same Auth0 accounts, shows only their apps, and writes a history entry with every change, naming the manager. No vendor dashboard and no git. |

### Why not A to E

The "no git" and "no vendor dashboard" rules mean **every** option needs
our own console for app managers; no vendor's UI can be handed to a
non-engineer safely. Once we build the console, the vendor only stores
and evaluates values, which D1 and a small Worker do for free.

- **A, Flagship**, stays the best vendor fit (it is Cloudflare), but its
  price after the beta is unknown, it doesn't run in our Rust Workers
  without a token or JS glue, and per-user values inside flag rules invite
  lost updates. Worth another look at GA (see "Revisit").
- **B, flagd**, keeps flags in files, which is what Jim ruled out.
- **C, Unleash or GrowthBook**, is a new vendor or a server to host, with a
  separate sign-in and seats for every manager.
- **D, PostHog**, shows managers analytics they shouldn't see, and can't
  serve Blue Eel.
- **E, Auth0 metadata**, needs a token that can edit every user, and
  changes wait for the next sign-in. Auth0 stays the home for **roles**
  (who manages which app), not for modes.

## Decision

Option F. Each item is a default, so Jim can answer "yes to all".

1. **Developers declare an app's modes in its code; app managers set
   them.** Each app lists its modes with the `flags` crate (item 5): a key,
   a plain-language name and description, the choices, and the default.
   Adding a mode is a code change, since code has to act on it. On every
   deploy, a CI step publishes the app's list of modes to the flags
   Worker, which adds new ones, updates names and descriptions, and marks
   ones no longer in the code as retired (kept, hidden). Everything an app
   manager changes, which choice each user gets and the default for
   everyone, lives only in D1 and never in git. A mode's default in code
   is used only until a manager sets one, and whenever the Worker can't be
   reached.
2. **One D1 database, `jimhillr-flags`**, created by the `infra`
   cloudflare stack. It holds runtime data a deployed service writes, which
   decision 0004 allows, and which the public repo couldn't hold anyway:
   - `modes (app, key, name, description, choices, code_default, retired)`
   - `settings (app, key, sub, choice, set_by, set_at, expires_at)`,
     where an empty `sub` is the setting for everyone in the app
   - `users (app, sub, email, name, last_seen)`, filled in when a signed-in
     user first opens the app, so managers can find people by name or
     email without a Management API token
   - `history (id, at, actor_sub, actor_email, app, key, sub, before,
     after, reason, request_id)`, append-only
3. **The `jimhillr-flags` Worker**, built with `workers-rs` and the `d1`
   feature:
   - `GET /v1/apps/{app}/modes` returns the caller's choice for every mode.
     It needs an Auth0 **access token**, verified against Auth0's public
     keys (signature, issuer, audience, expiry), and takes `sub` **only**
     from that token. Without a valid token it returns the defaults. CORS
     allows only that app's own origins.
   - `/v1/manage/*` (list modes, find users, set or clear a choice, read
     history) needs an access token whose `permissions` claim holds
     `manage:{app}` for the app in the path. Auth0 adds that claim from the
     manager's role (Auth0 role-based access control, on the free plan). A
     manager of cahokia gets a 403 on every other app.
   - `POST /v1/apps/{app}/catalog` is the deploy step's call from item 1.
     It takes a token held only by the deploy job (minted by Infra Mint,
     decision 0011), and it can only change the list of modes, never a
     setting.
   - Every change to `settings` and its `history` row are written in one
     D1 batch, so there is no change without a history entry. The Worker
     never updates or deletes `history`, and SQLite triggers reject it.
   - A choice must be one of the mode's declared choices; anything else is
     a 400.
4. **The console is its own Dioxus app, `jimhillr-console`** on Cloudflare
   Pages, signing in through `auth_sdk` like every other app. A manager
   sees the apps they manage; for each one, its modes in plain language;
   a search for a user by name or email; each mode's choice for that user
   with a dropdown to change it, a required "why" and an optional "until"
   date; the setting for everyone; and the full history of changes. It is
   written for someone who has never seen the code. Putting it in each
   app instead would mean building the same pages in every app; putting it
   in pane would mix it with Jim's developer dashboard, which managers
   must never see.
5. **A shared Rust crate, `flags`**, gives apps one small,
   OpenFeature-shaped API (`mode("editor")` returning the user's choice,
   with the default from code) for Dioxus apps and Workers alike, plus the
   list of modes the deploy step publishes. It fetches once per session,
   caches the answer, and falls back to the code defaults when the Worker
   can't be reached, so an outage never breaks an app. Keeping the
   OpenFeature shape means a later move to Flagship, or any OpenFeature
   provider, changes the crate and the Worker, not every app.
6. **Developers appoint app managers in Auth0, never in git.** Each app
   has an Auth0 role, `{app}-manager`, carrying the `manage:{app}`
   permission. The roles and permissions are IaC (the `infra` auth0
   stack), which is a credential change under decision 0010 and applies
   only after Jim's second approval. **Who holds a role** names a person,
   so it can't be in the public repo: Jim assigns it in the Auth0 dashboard,
   following a runbook added with phase 2, and a settings check in the
   weekly digest lists every manager so Jim can confirm the list. A
   console page for Jim to appoint managers can follow later.
7. **Modes never grant permissions.** A mode changes how an app looks or
   behaves for a person; it is never what lets someone read or change data
   they otherwise couldn't, and never makes someone a manager. Who may do
   what stays in Auth0 roles. Anything privileged a mode turns on is also
   checked on the server. A CI check refuses modes named like permissions
   (`admin`, `role`, `manager`, `superuser`, `bypass`).
8. **Developers see what managers do.** Pane shows each app's recent
   history (counts by mode, any setting with no end date), read through a
   read-only token, and the weekly human issues digest summarises the week.

## Threat model

Setting someone's modes is a privileged action, and the people doing it
are not engineers, so the flags Worker and the console each get a
`docs/threat.toml` when they are built. The main threats:

| STRIDE | Threat | Mitigation |
| --- | --- | --- |
| Spoofing | A user asks for another user's modes, or pretends to be a manager | `sub` and permissions come only from a verified Auth0 access token; `/v1/manage/*` checks `manage:{app}` for the app in the path |
| Spoofing | A manager's account is taken over (phished password, shared laptop) | Managers must use multi-factor sign-in, enforced by an Auth0 Action for anyone holding a manager role (the factors the free plan offers are confirmed in phase 2); short-lived access tokens; the console signs out after inactivity |
| Tampering | A manager changes an app they don't manage, or sets a value the app doesn't expect | Permission per app; choices must be one of the declared ones |
| Tampering | Two managers change the same setting at once | Each write names the value it replaces; a stale write gets a 409 and the console reloads |
| Tampering | An agent ships Worker or console code that weakens the checks | Agents can merge code, as with pane in decision 0010. Contained by item 7 (a mode can't grant access), the append-only history, pane and the digest showing every change (item 8), and code review on every PR touching the flags Worker or console (a protected path for the reviewer account) |
| Tampering | The deploy token is used to change settings | It can only publish the list of modes (item 3), never a setting |
| Repudiation | A manager denies a change | Every change has a history row written in the same batch, naming the manager from the verified token, with before and after, the reason and a request id; the table rejects updates and deletes; managers see the same history in the console |
| Information disclosure | A manager sees more about users than they need | The console shows only users of apps they manage, and only name, email, last seen and modes; nothing else about them is stored |
| Information disclosure | User ids or emails leak | They live only in D1, never in the repo, logs, pane's snapshot or error messages; `users` rows are deleted after a year without a visit |
| Denial of service | Flooding uses up the free 100,000 Worker requests a day | Apps fetch once per session; the Worker rate-limits per user and per IP; on any failure apps use code defaults, so the worst case is everyone gets defaults |
| Elevation of privilege | A mode is used as a back door to admin features, or a manager makes themselves or others managers | Item 7; manager roles are assigned only by Jim in Auth0 (item 6), and no Worker or console token can assign roles |

What it doesn't protect against: Jim's own Auth0 account being taken over
(use a passkey), and someone with D1 write access through the Cloudflare
deploy token editing the tables directly. The deploy token already has D1
Write (`jimhillr-deploy`); the weekly digest compares `settings` with
`history` and reports any setting with no matching history row.

## Phases

1. **The `flags` Worker, D1 schema and the `flags` crate**, with tests for
   token and permission checks, the batch write, stale writes and the
   append-only triggers, and the Worker's threat model and
   `docs/cost.toml`. About 2 PRs.
2. **IaC and Auth0**: the D1 database and Worker in the cloudflare stack;
   the Auth0 API, its permissions, the manager roles and the
   multi-factor Action in the auth0 stack (credential change, Jim's second
   approval); the catalog token in Infra Mint; the runbook for appointing
   a manager; `standards/settings.toml` entries. About 1 PR.
3. **The console**, `jimhillr-console`, with its user guide written for
   app managers, threat model and evals. About 2 PRs.
4. **First app: cahokia**, with one real mode, the deploy step, pane's
   history view, and an eval that checks the default and a manager's
   setting end to end. About 1 PR.

## Cost

$0 a month. One Worker (budget 25), one Pages project for the console
(budget 25), one D1 database (of 10) with a few hundred rows written a day
at most, and an Auth0 API and roles on the free plan. Each manager counts
as one of Auth0's free 25,000 monthly active users. No new vendor. One new
secret, the catalog token, minted and rotated by Infra Mint. Claude build
effort is about 6 PRs.

## Revisit

Look at Cloudflare Flagship again when it reaches general availability, if
its free plan covers our usage and `workers-rs` gains a binding: the
Worker could then store values in Flagship behind the same console. The
`flags` crate's OpenFeature shape keeps that change out of the apps.
Percentage rollouts and A/B splits, if we need them, are added to the
Worker first and are the other trigger to revisit.

## Consequences

- App managers change modes for one user, or everyone, in the console,
  with no git, PR or vendor dashboard; every change is in the history.
- Developers add a mode with a code change and a deploy; they never set a
  user's mode through git.
- The tech radar lists Flagship and OpenFeature in Assess, and flagd,
  Unleash and GrowthBook in Hold, with this record as the source.
- Apps gain one network call per session, cached, with defaults on failure.
