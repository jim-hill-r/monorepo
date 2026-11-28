# Plan: user feedback, support and in-app user guides, as platform libraries

Started 2026-11-28 from Jim's request:

> As a part of our technical platform we should create user feedback
> tooling. I think we already have some plans for a vercel like chrome
> plugin and some voting features in bear_down, but let's formalize this
> project a bit so we can embed it into the sparkhill webapp allowing users
> to provide feedback.

> The feedback tooling should also provide some ability to get support as
> well.

> I am requiring a user guide, but can we build into the technical platform
> a way to surface the user guides into applications.

And his answers to the first draft's three questions (2026-11-28):

> 1. Same origin. Assume products are owned by different companies during
>    architecture. We should have shared technical platform services, just
>    libraries and patterns.
> 2. We should integrate with some vendor that is easy to provision with
>    IAC or scripting and not email. I think something like a Google form
>    that we could then wire up to our developer platform might work.
>    Explore lots of options and suggest several
> 3. This is a really hard question. I think this opens up a big questions
>    about data warehousing and ensuring different products are generic and
>    don't use shared database resources without exploding our cost. I think
>    shared DB infra with tenatized tables is okay as it would be easy to
>    migrate to isolated tables if necessary. Explore all the options and
>    propose a good one

Status: **approved** by Jim (PR 915, 2026-11-28). Decisions:
[0020](../decisions/0020-user-feedback-and-support.md) (this component)
and [0021](../decisions/0021-tenant-scoped-product-data.md) (where product
data lives). The work items are issue files with `epic: User feedback`,
one per phase.

## Goal

A `feedback` project in the **technical-platform** tier (decision 0019)
that any product embeds so its users can:

1. **Get help**: read the product's own user guide (the
   `docs/user/README.md` every project must have, decision 0017) inside
   the app, offline.
2. **Ask for support**: send a question and read the answer, without
   giving an email address.
3. **Send feedback**: a short form (topic, optional rating, note) and
   one-tap votes like bear_down's Yes / No / Confusing.

Blue Eel (sparkhill/webapp) embeds it first, under its hardest rule,
[SPRK-006](../../../sparkhill/docs/requirements.md): nothing leaves the
device unless the user presses Send, only what they typed or picked is
sent, and there is no tracking, cookie, account or third-party script.

## The architecture rule: products are separate companies

Jim's rule for this design (answer 1): design as if each product belonged
to a different company. So the platform is **libraries and patterns, not
shared services**:

- Each product runs its own feedback and support endpoint, on its own
  origin, in its own Worker, built from the shared crates. No product's
  requests pass through another product's Worker or a central service.
- Each product owns its data. It sits in the shared D1 infrastructure in
  tables scoped to the product as a tenant, which can move to the
  product's own database with one export (decision 0021, answer 3).
- Each product answers its own users in its own management app (decision
  0018), built from the shared management app component.

What stays shared is code (crates), patterns (this plan), infrastructure
definitions (the `infra` stacks) and the one D1 database that holds
tenant-scoped tables.

## What already exists, and how it fits

| Piece | Where it is planned or built | Who uses it | What happens to it |
| --- | --- | --- | --- |
| Review toolbar: Vercel-like pins on a page, Jim only, sent to pane | [Eval engine plan](2026-11-eval-engine.md), Phase 6; decision 0003 item 8 | Jim and reviewers | Becomes the staff modes of the same Feedback button users see ([One feedback tool, four roles](#one-feedback-tool-four-roles)), built in Phase 8 with its own browser extension. Its eval engine issues moved to this epic (Jim, 2026-11-28). |
| Fan votes on each game, with an A/B test | [bear_down fan feedback plan](../../../bear_down/docs/plans/2026-11-fan-feedback.md), `bear_down/web/src/feedback.rs` | Public fans | The buttons become the shared `QuickVote` component, and the votes move to bear_down's own endpoint and tenant tables (Phase 7). |
| The `telemetry` Worker | [telemetry/](../../../telemetry/README.md) | Every app | Not used for user feedback or support: it is a shared service, which the rule above rules out for product data. Its validation and scrubbing code moves into `feedback/model` so products reuse it as a library. Whether telemetry itself becomes a library is a separate question for later. |

The user form and the staff modes share `feedback/model`, never data:
the user form takes no screenshot, reads no DOM and attaches no errors,
because a user can't check what those contain before sending.

## One feedback tool, four roles

Jim, 2026-11-28, on the eval review: "the vercel like plugin may fit in
better with the feedback project. Maybe it's a common tool but depending
on who is logged in, you can give user feedback, app manager feedback,
product owner feedback or engineering feedback?" So there is one Feedback
button, and who is signed in picks what it does:

| Role | Who | What they can send | Where it goes |
| --- | --- | --- | --- |
| User | Anyone, signed out | The privacy-safe form above: topic, rating, note; support requests | The product's own tenant tables and management app inbox |
| App manager | A product's managers (decision 0018) | User feedback plus pins on the page, the build, and a "for my users" flag | The product's own inbox |
| Product owner | Jim today; any requirements owner | Pins with kind (bug, idea, not met, looks good) and a requirement; ratings | `<project>/evals/feedback.jsonl` through pane, where they count as ratings (eval engine) |
| Engineering | Agents' reviewers and engineers | Pins with screenshot, last 20 console errors, selectors and build | `FIX` or `TODO` issue files in the project's `issues/`, through pane |

- **Signed out is the default and stays safe.** The staff code (pins,
  screenshots, console capture) is a separate module that loads only after
  a staff sign-in returns a role, so a user's page never runs it and
  SPRK-006 holds. The production builds of strict products can leave the
  module out entirely and rely on the browser extension (Phase 8).
- **Staff sign-in** is split by who signs in
  ([decision 0024](../decisions/0024-internal-sign-in.md)). App managers use
  the product's Auth0 sign-in, as the management apps do (decision 0018),
  so one product's managers can't send as another's. Product owners and
  engineers are internal: they sign in through pane's Cloudflare Access
  login, and their staff modes come only through the browser extension
  (Phase 8), which sends to pane. No product build carries their code.
- **Same origin for product data.** User and app manager feedback goes to
  the product's own `/api/feedback`. Product owner and engineering
  feedback is about how we build software, so it goes to pane, the
  software development platform, not to a product's tables.

## What a Blue Eel user sees

Blue Eel's learners are children, so help, support and feedback are for
the grown-up with them. They sit in the practice menu (PR 911), never on
the tracing screen:

- **Help** opens the app's user guide, built into the app, with its
  `## Getting help` section first.
- **Ask for help** opens a support form: a subject from a short list, the
  question (at most 2,000 characters), a preview of exactly what will be
  sent, and Send. The app then shows a private link to the request
  ("Keep this link to see our answer"), saved on this device. The answer
  appears there, and the menu shows a dot when it arrives. No email, no
  account.
- **Send feedback** opens a form: *What is it about?* (Blue Eel: *A letter
  or word*, *Checking my tracing*, *Settings*, *Something else*), an
  optional 1 to 5 rating, an optional note (at most 500 characters), the
  same preview, and Send. After Send: "Thanks, this goes only to the
  people who build Blue Eel."

Both forms say "Please don't include a child's name or contact details",
and scrub anything that looks like an email address or phone number. No
learner name, progress, tracing or setting is ever attached: the app
passes the widget only the topics, the page and the build.

## How it is built

All Rust. The browser side is framework-free at its core, so it fits
Dioxus 0.6 (sparkhill), 0.7 (bear_down, cookbook, pane, cahokia) and the
Perseus sites.

| Crate | Side | What it is |
| --- | --- | --- |
| `feedback/model` | Both | `UserFeedback`, `Vote`, `SupportRequest` and `SupportReply`, plus the review toolbar's `Comment` and `Anchor`. Validation and email and phone scrubbing (moved from telemetry). |
| `feedback/help` | Browser | In-app user guides (below). Usable on its own. |
| `feedback/widget` | Browser | Form state, the exact preview, one send per 30 seconds, `web-sys` `fetch` to a same-origin path, and the support request link kept in `localStorage`. |
| `feedback/dioxus` | Browser | `HelpPanel`, `HelpLink`, `SupportForm`, `SupportThread`, `FeedbackForm`, `QuickVote`. A feature per Dioxus line (`dioxus06`, `dioxus07`). Styled only with design_system custom properties. |
| `feedback/worker` | Server | A `workers-rs` handler a product mounts at `/api/feedback` and `/api/support`: validates with `feedback/model`, rate-limits with the product's own rate-limit binding, drops synthetic traffic, and writes through `tenant_store`. |
| `feedback/manage` | Server and browser | The Feedback and Support inbox pages for the management app component of decision 0018: list, filter, reply, close, and export. |
| `tenant_store` | Server | The tenant-scoped D1 library from [decision 0021](../decisions/0021-tenant-scoped-product-data.md). A separate technical-platform project, since every product's runtime data uses it, not only feedback. |

### In-app user guides

Every project already writes its user guide (decision 0017), so the
platform shows that same guide inside the app. One source, two places.

- **At build time**, `feedback/help` reads `<project>/docs/user/*.md`
  (`feedback_help::embed!("../docs/user")`) and turns each page into
  sections with headings, anchors and plain HTML (`pulldown-cmark`, raw
  HTML and remote images dropped). Compiled into the app, so help works
  offline and fetches nothing.
- **In the app**, `HelpPanel` shows the guide with `## Getting help`
  first, a search over headings, and deep links; `HelpLink` is the small
  "?" an app puts beside a setting to open help at its section.
- **Links stay true.** Links between guide pages open in the panel; links
  to the repo open in a new tab; a link to a missing heading fails the
  build.
- **Always current.** The guide ships with each release, so it matches
  the build the user runs.
- **Other hosts.** Perseus sites and Bevy games can call the same
  Markdown-to-sections function and render the sections their own way.

### Where requests go: the product's own origin

The widget posts to `/api/feedback` and `/api/support` on the product's
own origin, so a CSP of `connect-src 'self'` holds (answer 1). For Blue
Eel that means a Worker of its own:

- **Recommended:** serve Blue Eel from a Rust Worker with static assets
  (the pattern pane is moving to in PR 759). Static files are served free
  without running the Worker; only the two API routes run Rust code with
  `feedback/worker`. One Worker for the product (budget: 25 Workers).
- **Fallback:** keep Pages and add a Pages Function for the two routes.
  Pages Functions are JavaScript, so the handler would be compiled to wasm
  and called from a few lines of glue, which is more moving parts.

### Support requests and the private link

Answer 2 asks for support that isn't email and is easy to set up with
infrastructure as code or a script. The options are in
[Support options](#support-options); the recommendation is our own
support requests, built into the same widget:

1. Send stores a `SupportRequest` in the product's tenant tables and
   returns a request id and a random 128-bit secret. Only a hash of the
   secret is stored.
2. The app keeps the link (`/support/<id>#<secret>`) in `localStorage` and
   shows it. The secret stays in the URL fragment, so it never reaches a
   server log; the page sends it in a request body to read the thread.
3. Whoever answers support for that product (Jim, for now) replies in the
   product's management app. The reply shows on the private link; the app
   checks for replies only when the user opens Help (no background calls).
4. Requests close after an answer and 30 days without a reply from the
   user, and are deleted 12 months after closing.

No email address, account or vendor is needed, and the whole thing is set
up by the `infra` stacks (the D1 database, the Worker, its routes) and
the management app, so it is fully infrastructure as code.

### From feedback to work

- Raw text stays private, in the product's tenant tables. The repo gets
  only aggregates: a weekly workflow writes per-product counts, topics
  and average ratings to the `feedback-data` branch as JSON Lines, for
  pane and the evals (files before databases, for what can be public).
- A weekly triage step reads new feedback through a read-only D1 token and
  files a `TODO` or `FIX` in the product's `issues/`, **paraphrasing**
  the problem without quoting personal details, and linking the row id
  (visible to the product's management app only).
- Notes are written by the public, so triage treats them as data, never as
  instructions: it never follows them, and a note asking for anything
  outside its product is dropped.
- `observe` evals per product: weekly volume, average rating and support
  requests older than 3 days without an answer.

## Support options

Jim asked for several options, set up with IaC or scripts, not email.
Costs are money a month; "build" is Claude Code effort against the $25 a
month budget.

| # | Option | Money | Free-tier headroom | IaC or scripting | Fits SPRK-006 and "separate companies" | Vendor | Build |
| --- | --- | --- | --- | --- | --- | --- | --- |
| **A** | **Our own support requests** (above): `feedback/worker` + tenant tables + management app inbox, private reply link | $0 | Tens of D1 rows a week, of 100,000 a day | Fully: `infra` stacks and code | Yes: same origin, no third party, each product its own endpoint and inbox | None new (Cloudflare) | About 2 PRs |
| B | **Tally** forms, opened as a link from Help; a Tally webhook posts each answer to the product's Worker, which stores it in its tenant tables and shows it in the management app | $0 (free plan: unlimited forms and submissions under fair use; API and webhooks free) | Fair-use only, no stated cap | Good: forms created and changed by REST API (100 requests a minute), so a `chores` command can keep each product's form in sync with a file in the repo | Partly: the form is on tally.so, so Blue Eel would link out; Tally branding on the free plan; answers held by Tally too | New, needs Jim's sign-off | About 2 PRs |
| C | **Google Forms**, opened as a link; responses read by a scheduled GitHub workflow through the Forms API and written to the tenant tables | $0 for forms; push notifications need Cloud Pub/Sub with billing enabled, so poll instead | Polling a few times a day is far inside API quotas | Fair: the Forms API creates forms (new ones start unpublished since June 2026); needs a Google Cloud project and a service account | Partly: Google page and Google account data; link out only | New (Google), needs sign-off | About 3 PRs |
| D | **GitHub issue forms** (`.github/ISSUE_TEMPLATE/*.yml`), a "Get help" link per product | $0 | Unlimited on a public repo | Excellent: the forms are YAML files in the repo | No for Blue Eel: users need a GitHub account and everything is public | GitHub (approved) | Under 1 PR |
| E | **GitHub Discussions**, a Q&A category per product | $0 | Unlimited | Good: categories by API or the `github` IaC stack | No for Blue Eel (account, public); fine for developer-facing tools like auth_sdk or chores | GitHub (approved) | Under 1 PR |
| F | **Jira Service Management** free (3 agents, customer portal, forms) | $0 | 3 agents | Fair: REST API; community Terraform providers only | Portal on atlassian.net; users get an Atlassian account | New, needs sign-off | About 2 PRs |
| G | **Zoho Desk** free (3 agents, web forms) | $0 | 3 agents | Weak: email-centred, API not on the free plan as far as we found | Third-party portal and widget script | New, needs sign-off | About 2 PRs |
| H | Hosted chat (Crisp, Tawk.to) | $0 on their free plans | Small seat limits | Weak: dashboards | No: a third-party script on the page | New, needs sign-off | About 1 PR |

Ruled out: email (Jim), Freshdesk (its free plan is now time-limited),
self-hosted osTicket or HESK (they need a PHP server we don't run), and
anything that loads a script from another site into Blue Eel.

**Chosen (Jim, 2026-11-28): A for every product**, because it is the only option
that is same-origin, keeps each product's users and data separate, needs
no new vendor and is defined entirely in code. **B (Tally)** is the best
vendor option if Jim wants support forms that a non-engineer edits
without a PR: it is free, its API makes forms scriptable, and its webhook
lands answers in the same tenant tables and management app as A, so a
product can start on B and move to A (or back) without changing its
inbox. **E** is a good extra for developer-facing projects.

## Data: where product data lives

Answer 3 is about more than feedback: every product that stores runtime
data (feedback, support, feature flags, future accounts) needs a rule that
keeps products generic and separate without paying for a database each.
That rule is [decision 0021](../decisions/0021-tenant-scoped-product-data.md),
summarised here:

- **One shared D1 database, `jimhillr-products`**, holds every product's
  runtime data. Every table has `tenant` (the product's id) as the first
  column of its primary key and of every index.
- **Only `tenant_store` talks to it.** A product's Worker opens a
  `TenantStore` for its own tenant, which is fixed at deploy time from the
  Worker's configuration; every query the library sends is scoped to that
  tenant, and a test fails any query string without the tenant
  condition. Products never write raw SQL against the shared database.
- **Per-tenant budgets.** Each tenant has a daily write budget and a
  storage budget in `standards/radar/limits.toml` (a share of D1's 100,000
  rows a day and 5 GB), enforced by `tenant_store`: past the budget it
  refuses writes politely, it never asks for more.
- **Moving out is one command.** `chores tenant-export <tenant>` dumps one
  tenant's rows as a `.sql` file that builds the same tables in a new
  database; switching the product's binding moves it. A product moves out
  when it passes 100 MB, needs its own backups or region, or is handed to
  another owner.
- **Warehouse later, files now.** Cross-product reporting reads the
  weekly aggregate files on the `feedback-data` branch. A real warehouse
  (Cloudflare's Basin Catalog, the R2 Iceberg tables) needs a payment
  card on the account, so it is ruled out by the free-tier rule until Jim
  decides otherwise.

## Abuse and limits

- Each product's Worker has its own rate-limit binding (10 requests per
  10 seconds per IP, as telemetry uses), a 4 KB body cap for feedback and
  8 KB for support, and the widget's one send per 30 seconds.
- `tenant_store`'s per-tenant write budget caps the worst case at that
  product's share, so one spammed product can't use another's quota.
- No CAPTCHA: Turnstile loads a script from another site, which SPRK-006
  forbids. If spam appears: a honeypot field, then a lower budget, then a
  decision for Jim.
- Synthetic traffic (`jimhillr-evals` user agent, `x-jimhillr-synthetic`)
  is answered and not stored.

## Requirements

Phase 1 creates `feedback/docs/requirements.md` with these, for Jim to
approve (prefix `FDBK`):

- **FDBK-001: Nothing is sent until the user presses Send.** Must. Why:
  our apps promise that nothing leaves the device on its own. Accept:
  opening help or a form, typing and cancelling make no request; Send
  makes exactly one; reading a support answer happens only when the user
  opens Help.
- **FDBK-002: The user sees exactly what is sent.** Must. Why: consent
  means knowing what you agree to. Accept: the preview shows every field;
  the request body equals the preview.
- **FDBK-003: Only what the user typed or picked is sent.** Must. Why: no
  tracking (SPRK-006). Accept: feedback holds only product, build, page
  (no query string), topic, rating and note; support adds only subject
  and question; no id, cookie, storage value, screenshot, DOM or error
  text; no IP address is stored; emails and phone numbers are replaced.
- **FDBK-004: It works under a same-origin-only CSP.** Must. Why: Blue
  Eel's headers allow `connect-src 'self'` only. Accept: every request
  goes to the page's own origin; no inline script, eval or style beyond
  what the host already allows.
- **FDBK-005: Help works offline.** Should. Accept: the help panel shows
  the product's user guide with the network off.
- **FDBK-006: Feedback becomes work.** Should. Why: feedback nobody reads
  is worse than none. Accept: each week's aggregates are on the
  `feedback-data` branch within 8 days, and each actionable item has an
  issue file within 8 more.
- **FDBK-007: Any of our web apps can embed it in a few lines.** Should.
  Accept: a Dioxus 0.6 and a Dioxus 0.7 app each add help, support and
  feedback with one dependency and under 20 lines of UI code, plus
  mounting `feedback/worker`; both are covered by Playwright tests.
- **FDBK-008: Everyone can use it.** Should. Accept: every control has a
  label, works by keyboard, meets WCAG 2.2 AA in every design_system theme
  and mode, and fits a 360 px wide screen.
- **FDBK-009: An app shows its own user guide.** Should. Why: Jim
  (2026-11-28). Accept: every page of `docs/user/` shows in the app,
  identical to the committed Markdown, with working links and section deep
  links; a broken internal link fails the build.
- **FDBK-010: A user can get an answer without giving contact details.**
  Should. Why: Jim wants support, and SPRK-006 rules out collecting
  contact details. Accept: a support request returns a private link that
  shows the request and every reply; replies written in the management
  app appear there; without the link no one can read the request.
- **FDBK-011: One product can never read or fill another's data.** Must.
  Why: products are designed as separate companies (Jim, 2026-11-28).
  Accept: every `tenant_store` query is scoped to the caller's tenant
  (a test checks each query), a tenant past its write budget is refused
  without affecting others, and `tenant-export` of one tenant contains
  no other tenant's rows.

And for Blue Eel, in Phase 6's requirement PR (Help alone may ship in
Phase 3):

- **SPRK-011: A grown-up can get help, ask for support and send
  feedback.** Should. Accept: the practice menu has Help, Ask for help
  and Send feedback; they meet FDBK-001 to FDBK-004 and FDBK-010, and
  never include a learner's name, progress or tracing.
- **SPRK-006** gains one clause: "except what the user reads and sends
  from Help, to the app's own origin" (its CSP part is unchanged).

## Cost

All $0 on approved vendors (Cloudflare, GitHub) and free plans
([free-tiers.md](../free-tiers.md)), with option A for support:

| Piece | Free limit | Expected use |
| --- | --- | --- |
| A Worker per product that embeds it (Blue Eel first) | 25 Workers budgeted; 100,000 requests a day, shared | 1 Worker for Blue Eel; static assets don't count as requests |
| D1 `jimhillr-products` | 10 databases (1 used here); 5 GB; 100,000 rows written and 5,000,000 read a day | Tens of rows a week; per-tenant budgets cap the worst case |
| Rate-limit bindings | Free | One per product Worker |
| Weekly aggregate export (GitHub Actions) | Free on a public repo | About a minute a week |
| Browser extension (Phase 8) | Unpacked from a GitHub release: free | Avoids the Chrome Web Store's $5 fee |

Option B (Tally) would add $0 and a vendor; C (Google Forms) $0, a vendor
and a Google Cloud project. Claude Code build effort: about nine medium
agent PRs, one per phase. No new vendor with option A.

## Phases

Each phase is one PR and one issue file (`epic: User feedback`).

1. **Project and model.** The `feedback` project (mise.toml, README,
   CONTRIBUTING, user guide, cost and threat files, FDBK-001 to 011,
   evals.toml) and `feedback/model`, with telemetry's scrubbing moved in.
   Replaces the eval engine's `eval-engine-phase-6-app-feedback` issue.
2. **Tenant store.** Decision 0021's `tenant_store` project, the
   `jimhillr-products` D1 database in the `infra` cloudflare stack, the
   per-tenant budgets in `limits.toml` and `free-tiers.sh`, and
   `chores tenant-export`.
3. **In-app user guides.** `feedback/help`, `HelpPanel` and `HelpLink`;
   then Blue Eel's Help page. Sends nothing, so it ships first.
4. **Feedback widget and worker.** `feedback/widget`, `FeedbackForm` and
   `feedback/worker`'s `/api/feedback`, with Playwright tests for
   FDBK-001 to 004.
5. **Support requests.** `SupportForm`, `SupportThread`, `/api/support`,
   the private link, and `feedback/manage`'s inbox in the management app
   component (needs decision 0018's component).
6. **Blue Eel.** Serve it from a Rust Worker with static assets, mount
   `feedback/worker`, add Ask for help and Send feedback to the practice
   menu, SPRK-011 and the SPRK-006 clause (needs-human), and evals.
7. **QuickVote and bear_down.** `QuickVote`, and bear_down's votes moved
   to its own endpoint and tenant tables, keeping its A/B arms.
8. **One feedback tool with roles.** The build meta tag, staff sign-in
   with roles, the staff modes (the review toolbar) in cookbook first and
   then every web app, the cahokia webviews and the Bevy plugin, and the
   Manifest V3 extension that loads the staff modes on production sites
   (unpacked from a GitHub release). Needs pane's feedback routes (eval
   engine Phase 5) for the product owner and engineering roles.
9. **Triage and pane.** The weekly aggregate export, triage into issue
   files, pane's Feedback page showing aggregates, and the `observe`
   evals.

## Questions for Jim

1. **Data rule.** Recommended: one shared D1 with tenant-scoped tables and
   per-tenant budgets (decision 0021; $0, 1 of 10 databases, about 1 PR).
   Alternatives are in that decision: a database per product, Durable
   Objects per tenant, or a hosted Postgres with row-level security.
2. **Blue Eel's server.** Recommended: move it to a Rust Worker with static
   assets ($0, 1 of 25 Workers, part of Phase 6). Alternative: a Pages
   Function with wasm glue ($0, more moving parts).

## Decisions

- 2026-11-28, Jim: same origin; products are designed as separate
  companies and the platform shares only libraries and patterns.
- 2026-11-28, Jim: support through something set up with IaC or scripts,
  not email; options above.
- 2026-11-28, Jim (eval review): the Vercel-like toolbar belongs here,
  as one tool whose signed-in role picks user, app manager, product owner
  or engineering feedback.
- 2026-11-28, Jim (decision card): option A, our own support requests, for
  every product. Tally and the other vendors are not used.
- 2026-11-28, Jim: shared database infrastructure with tenant-scoped
  tables is acceptable if moving a tenant out stays easy; decision 0021.
