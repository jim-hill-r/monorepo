# 0023. Observability as one project and review domain: Cloudflare for signals and logs, a watchdog Worker for checks and alert rules, GitHub Mobile to page Jim's phone

- Status: Accepted
- Decided by: Jim (2026-11-28, chose "GitHub Mobile only" on the pager decision card, over PagerDuty Free, Pushover and ntfy.sh; the rest as recommended; on 2026-11-28 made observability one project and review domain)
- Date: 2026-11-28
- Asked by: Jim
- Builds on: [0001](0001-deploy-feedback-loop.md) (Cloudflare is the
  observability backend, PostHog only for replay),
  [0003](0003-eval-engine.md) (production evals),
  [0007](0007-infrastructure-as-code.md) (IaC),
  [0010](0010-credential-second-approval.md) and
  [0011](0011-mint-secrets-by-workflow.md) (credentials),
  [0019](0019-review-roles.md) (review roles, which this record extends
  with an `observability` domain)

## Context

Jim (2026-11-28):

> We need to do tech selection on monitoring, alerting, logging and paging
> solutions. Lets look for many options and decide on the best one.

and, once the choices were made:

> All of these decisions can be lumped into a single domain called
> observability. We can make observability a first class citizen in our
> dev platform and have a reviewer for it and have all this work exist
> within an observability project.

The ask also said paging must reach Jim's phone, setup should be IaC or
scripts, and cost is weighed against the free tiers.

### What we already have

Decision 0001 chose Cloudflare as the observability backend and PostHog
(US region) only for real-user replay. Today that gives us:

- **Logs:** Workers Logs and Workers Observability for every Worker
  (`telemetry`, `tofu_state`, and pane's API once it lands).
- **Events:** the `telemetry` Worker, which takes browser errors, game,
  CLI and deploy events into Analytics Engine.
- **Real-user analytics:** Cloudflare Web Analytics and PostHog replay.
- **Checks on a schedule:** nightly CI and CD, production evals after each
  deploy (0003), the weekly free-tier usage and billing checks.

What is missing is the last mile. Nothing watches production between
nightly runs, nothing turns a red signal into an alert within minutes, and
nothing reaches Jim when he is away from GitHub. The deploy-feedback plan
said the weekly review "replaces a separate alerting tool"; that is a week
of latency for an outage.

### What we run

Static Dioxus web apps on Cloudflare Pages (cookbook, cahokia, pane,
sparkhill, blueeel, bear_down, the games' web builds), Rust Workers
(`telemetry`, `tofu_state`, the planned flags Worker of 0018 on D1), D1,
and PostHog in the browser. All on free plans with no card. Traffic is
small, there is one person to page, and most work is done by agents.

### What a solution must do

1. **Monitoring:** check every production URL and Worker from outside, a
   few times an hour, and read the error and deploy signals we already
   collect.
2. **Alerting:** rules written as code in the repo, deduplicated (one page
   per outage, not one per check), resolved automatically when the check
   passes again, and split into "wake Jim" and "tell Jim later".
3. **Logging:** structured logs from Workers and errors from browsers,
   queryable for a week, with anything worth keeping moved to files.
4. **Paging:** reach Jim's phone with a push, ideally one he
   acknowledges and that escalates to a call if he doesn't. (Jim chose a
   push with no escalation, to add no vendor.)
5. **$0 a month, free plans with no card, set up by IaC or scripts,** with
   as few new vendors as possible and good Claude support.

## Options

Free-plan facts were read from each vendor's pricing or docs pages on
2026-11-28; check them again before relying on a number.

### Logging

| Option | Free plan | Fit |
| --- | --- | --- |
| **Cloudflare Workers Logs and Observability** (have it) | From 2026-12-01: 0.5 GB a day ingested, kept 7 days; nothing past it on Free | Already on, no new vendor, query builder and an MCP server. **Chosen.** |
| Workers Logpush or OpenTelemetry export to another tool | Needs Workers Paid ($5 a month) | Breaks the free-tier rule. |
| Grafana Cloud Logs (Loki) | 50 GB a month, 14 days, no card | Good, but getting Worker logs there needs Logpush or OTLP export (paid) or a Tail Worker forwarding every event, which spends Worker requests. |
| Better Stack Logs | 3 GB, 3 days | Less than we have. |
| Axiom, New Relic, Datadog free tiers | Generous ingest (Axiom and New Relic) or 1-day retention (Datadog) | Same export problem as Grafana, plus a new vendor. |
| Sentry (errors) | Developer plan, one user | On Hold since 0001; PostHog's free 100,000 exceptions a month already cover browser errors if `telemetry` is not enough. |

### Monitoring (uptime and synthetic checks)

| Option | Free plan | Fit |
| --- | --- | --- |
| **A `watchdog` Rust Worker on one Cron Trigger** | Workers Free: 1 of our 3 budgeted Cron Triggers, about 3,000 requests a day of 100,000 | Checks every 5 minutes from Cloudflare's edge, reads Analytics Engine for error spikes after deploy events, rules in the repo, deployed like `telemetry`. No new vendor. **Chosen.** |
| Cloudflare Health Checks | Pro plan and up | Not on Free. |
| Cloudflare Monitors (SQL alerts on Workers events, threshold, anomaly or SLO; beta) | Plan not stated yet | Promising for Worker error rates with no code. Assess in phase 1; if it is free, it replaces the watchdog's error-spike rule. |
| Upptime or a scheduled GitHub Actions check | Free on a public repo | Scheduled workflows start late (often 5 to 15 minutes, sometimes skipped) and every run commits or spends a runner. Fine as the watchdog's own watchdog (below), not as the main monitor. |
| Grafana Cloud Synthetic Monitoring | 100,000 API checks a month | Works, but a second place for rules and a new vendor for something one Worker does. |
| Better Stack Uptime | 10 monitors, 1 status page, alerts by email and Slack only | Too few monitors, and no phone on Free. |
| UptimeRobot, Healthchecks.io | 50 monitors at 5 minutes for personal use (UptimeRobot, from memory, not rechecked); 20 heartbeat checks (Healthchecks.io) | Healthchecks.io's free plan has no SMS or phone. Both add a vendor for a job the Worker covers. |
| Self-hosted Uptime Kuma, Gatus, Prometheus and Alertmanager | Open source | Each needs a long-running server; no approved vendor hosts one for free. |

### Alerting

| Option | Fit |
| --- | --- |
| **Rules as code in the watchdog, deduplicated by the open issue** | Each check has a stable key. The watchdog opens an issue when a check fails twice in a row and closes it when the check passes; an open issue with that key in its title means "already alerted", so one outage is one issue and the Worker keeps no alert state of its own. **Chosen.** |
| Cloudflare Notifications | Free for email and webhooks on every plan (up to 100 webhook destinations); PagerDuty as a destination needs Business. Pages "Deployment failed" is free. **Used as an input:** its webhook points at the watchdog, which turns it into an alert. |
| Grafana Alerting | Good, but only sees what is in Grafana. |
| GitHub Actions failure notifications | Already email and GitHub Mobile; kept for CI. Nightly CD and Infra apply failures also send a low-urgency alert. |

### Paging

| Option | Cost | Phone reach | Fit |
| --- | --- | --- | --- |
| PagerDuty Free | $0, 5 users, 1 escalation policy, 1 schedule | App push, email, and 100 SMS and phone calls a month | Real paging: acknowledge, escalate, re-notify, high and low urgency, Events API v2 with dedup keys, an official Terraform provider and MCP server. Recommended, not chosen: it adds a vendor. First to revisit if pages are missed. |
| ntfy.sh | $0, no account | Android push can override Do Not Disturb if the user allows it; the iOS app calls itself "bare bones" | Simple, but best effort on one server, no acknowledge or escalation, and its terms say it must not be the "sole or primary means" of alerts for critical infrastructure. Topic names act as the password. |
| Pushover | $4.99 once per platform after a 30-day trial | Emergency priority repeats until acknowledged; high priority bypasses quiet hours | The best phone experience for one person, but it costs money and needs a card, so it breaks the free-tier rule without Jim's say-so. Second choice. |
| Grafana IRM (OnCall) | $0, 3 IRM users | Mobile app; phone and SMS on Free not confirmed | Fine if we were already on Grafana; we aren't. |
| Better Stack on-call | Phone and SMS need a $29 a month seat | | Paid. |
| Opsgenie | | | Atlassian has stopped selling it and is retiring it. |
| **GitHub Mobile notifications** (an alert opens a GitHub issue that mentions and assigns Jim) | $0, approved vendor | Push for mentions and assignments; the phone's own Focus or Do Not Disturb settings can let GitHub through, but nothing escalates to a call | No new vendor. **Chosen by Jim.** |
| Email from a Cloudflare Email Worker, Telegram or Discord bots | $0 | Push at the app's normal level | No acknowledge or escalation. |

### Whole-stack bundles considered

| Bundle | Money | New vendors | Why not, or why |
| --- | --- | --- | --- |
| A. Cloudflare + watchdog Worker + PagerDuty Free | $0 | 1 (PagerDuty) | Recommended first: real acknowledge and escalation. Jim chose not to add the vendor. |
| B. Cloudflare + watchdog Worker + ntfy.sh | $0 | 1 (ntfy) | No acknowledgement or escalation, and its terms rule out critical-alert use. |
| C. Cloudflare + watchdog Worker + Pushover | $4.99 once per phone platform | 1 (Pushover) | Best push, but needs a card and Jim raising a budget from $0. |
| D. GitHub only (Upptime + GitHub Mobile) | $0 | 0 | Scheduled workflows start late, so Upptime is too slow as the monitor. |
| **G. Cloudflare + watchdog Worker + GitHub Mobile** | $0 | 0 | **Chosen** (Jim, 2026-11-28): bundle A's monitoring with bundle D's pager. |
| E. Grafana Cloud Free for all four | $0 | 1 (Grafana) | Worker logs can't reach it without paid export, so logs stay split anyway; 3-user cap; a second rule store. |
| F. Better Stack Free | $0 for email only | 1 | No phone or SMS without a $29 a month seat. |

## Decision

Bundle G, in four parts, owned by one project and one review domain (below).

1. **Logging stays on Cloudflare.** Workers log structured JSON through
   `tracing` (one line per request with `project`, `build`, route, status
   and duration, never a user's personal data), which Workers Logs indexes
   and Workers Observability queries for 7 days. Browsers keep sending
   errors to `telemetry`. Anything worth keeping past a week becomes an
   issue file or a committed export, as 0001 already says. No log shipping
   to another vendor.
2. **Monitoring is the watchdog,** a Rust Worker in the `observability`
   project on one Cron Trigger that runs every 5 minutes. It reads its
   checks from `observability/checks.toml`, which an audit check keeps in step with every
   project's production `DEPLOY_URL` and the Workers in
   `standards/resources.toml`, so a new site can't ship unwatched. Each
   check is an HTTP request with an expected status and, where an eval
   already defines one, a content marker. A second rule asks Analytics
   Engine whether errors per project rose after its latest `deploy`
   event. Each run writes its results to Analytics Engine, so pane's
   Health page and the weekly review can chart them. Requests carry the
   `jimhillr-evals` user agent, so `telemetry` and PostHog ignore them.
   An hourly GitHub Actions job checks that the watchdog ran in the last
   15 minutes and pages if not, so the watchdog is itself watched.
3. **Alerting is rules as code, with two urgencies.**
   - **High urgency (pushes to Jim's phone):** a production site or Worker
     failing two checks in a row (about 10 minutes down), a fivefold error
     rise after a deploy, the watchdog silent for an hour, and a free-tier
     limit that will stop a service today (for example Workers requests
     past 80% before 00:00 UTC).
   - **Low urgency (an issue with no mention, so no push):** nightly CD,
     Infra apply or production eval failures, Cloudflare Notifications
     such as Pages "Deployment failed", and free-tier usage past its
     warning level.
   - Each alert has a stable key (`<project>/<check>`) at the start of its
     issue title. The watchdog opens the issue on failure, comments if the
     failure changes, and closes it on recovery, so one outage is one issue.
     Alert text names the project, check, URL and status, and never
     includes user data, request bodies or secrets.
   - Cloudflare Notifications that are free (email, webhooks) are turned
     on through IaC and point their webhook at the watchdog's `/notify`
     route, which checks the shared webhook secret and raises the alert.
4. **Paging is GitHub Mobile.** An alert is a GitHub issue in
   `jimhillr/monorepo` labelled `incident`, opened by a small GitHub App
   (`jimhillr-alerts`, Issues read and write on this one repository and
   nothing else). A high-urgency alert assigns Jim's own GitHub account
   and mentions it in the body, so GitHub Mobile pushes it to his phone;
   a low-urgency one does neither. Jim watches the repository as
   "Participating and @mentions", so only high-urgency alerts push, and
   lets GitHub Mobile through his phone's Focus or Do Not Disturb
   settings if he wants night pages (a runbook). Email of the same
   notification is the backup channel.
   - **Incident issues are alerts, not work.** This is the one use of
     GitHub issues in the repo: the watchdog opens and closes them, and
     any follow-up work becomes an issue file in the project's `issues/`
     folder as usual (decision 0006).
   - Acknowledging is closing the push; nothing re-notifies. To make up
     for that, an alert that stays open for an hour gets one more comment
     that mentions Jim again, and then no more until it changes.

### Observability is a project and a review domain

Jim: "make observability a first class citizen in our dev platform and
have a reviewer for it."

- **The `observability` project** holds all of this work: the watchdog
  Worker, its checks and alert rules, the alerts App manifest, the shared
  Worker logging setup, the runbooks' code side, its user guide and its
  issue files (epic "Observability"). It starts as a docs project with the
  phase issues and becomes a Rust Worker project in phase 1. It is in the
  `software-development-platform` tier of decision 0019, with `pane`,
  `evals` and `infra`, because it guards how we run software, so its
  changes get strict review.
- **`telemetry` moves into it later.** The events Worker of decision 0001
  is observability too. Moving its folder to `observability/telemetry`
  keeps the Worker's name and URL, so nothing is retired; it is a backlog
  issue rather than part of this change, so the move gets its own review.
- **A new review domain, `observability`,** joins the domain table of
  decision 0019, with one role, `observability-reviewer`, held by Jim over
  the whole repo for now (like every other domain role). A PR touches it
  when it changes:
  - anything in `observability/` (checks, alert rules, urgency, who is
    paged) or in `telemetry/`;
  - a Worker's `[observability]` settings in a `wrangler.toml`, or the
    shared logging setup;
  - a project's production `DEPLOY_URL`, which changes what is watched.
  The trigger is by path, so `chores gate` can detect it without
  judgment. Removing or loosening a check is the change this role exists
  to catch: a quiet monitor looks the same as a healthy site.
- The matrix row is the same as the others today: Jim in every tier.

### Setup by IaC and scripts

- The watchdog deploys like `telemetry` (`mise run deploy`,
  `free-tiers.sh worker`, the shared `cloudflare-worker.toml` tasks); its
  Cron Trigger counts against the FT-004 budget (1 of 3).
- The `infra` project's `github` stack gains the `incident` label, and
  its `cloudflare` stack the Cloudflare notification policies and their
  webhook destination. This needs terrars bindings for
  `cloudflare_notification_policy` and its webhooks, generated with the
  pinned terrars version.
- **The alerts App is made by a person.** GitHub has no API to create an
  App, so Jim creates `jimhillr-alerts` from a manifest the repo holds
  (a human task with a runbook) and installs it on this repository only.
  A workflow in the `infra-credentials` environment then copies its
  private key from a GitHub secret into the watchdog's Worker secret, so
  the key never passes through git, state or a chat, and each run needs
  Jim's second approval (0010). The Worker signs a short-lived
  installation token from it for each alert.
- **No secret in state.** The webhook secret for `/notify` is minted by
  Infra Mint (0011) straight into a Worker secret and a GitHub secret, and
  the notification policy reads it from the workflow's environment, so
  IAC-002 holds.

## Threat model

| Threat | Mitigation |
| --- | --- |
| A leaked alerts App key lets someone open issues and ping Jim | The App can only read and write issues on this repository; the key lives only in a GitHub secret and a Worker secret; rotating it is a new key in the App's settings and one approved workflow run, reminded quarterly by a settings check. |
| A forged Cloudflare webhook to `/notify` | Checked against the webhook secret Cloudflare sends; the route only forwards known notification types; rate-limited like `telemetry`. |
| Alert text leaks user data in a public issue | The repository is public, so alerts carry only project, check, URL, status and counts, never request bodies, user data, internal hostnames or secrets; a unit test rejects other fields. |
| A page is missed (phone on silent, GitHub push delayed) | Email backup, a second mention after an hour, and every open incident shows in pane and the weekly digest. If a real outage goes unseen, revisit PagerDuty Free. |
| The watchdog dies silently | The hourly Actions heartbeat check pages. |
| Alert fatigue makes Jim mute it | Only outages and fast-moving free-tier stops page; everything else is low urgency; each page's rule is reviewed in the weekly review. |

## Phases

1. **Watchdog Worker** in `observability/` with HTTP checks, `checks.toml`, the audit check
   against `DEPLOY_URL`s, results in Analytics Engine, alerts to the log
   only. Find out whether Cloudflare Monitors are on the Free plan.
2. **Paging:** Jim's human task to create the `jimhillr-alerts` App and
   set up GitHub Mobile (with a runbook), the `incident` label in IaC, the
   key-copy workflow, then the watchdog opens and closes incident issues.
   A test page on a fire drill proves the path to Jim's phone.
3. **Signals:** the error-spike rule, free-tier stop rules, the hourly
   heartbeat job, low-urgency alerts from nightly CD, Infra apply and
   production evals, and Cloudflare Notifications through `/notify`.
4. **Logging hygiene:** a shared `tracing` setup for Workers that logs one
   JSON line per request with no personal data, and the radar and
   `cost.toml` updates for every project the watchdog covers.

## Cost

$0 a month, and no card anywhere.

| Piece | Free limit | Our use |
| --- | --- | --- |
| Watchdog Cron Trigger | 5 per account (budget 3) | 1 |
| Workers requests | 100,000 a day | About 3,200 a day (288 runs, about 10 checks each) |
| Analytics Engine | 100,000 writes and 10,000 queries a day | About 3,000 writes and 300 queries a day |
| Workers Logs | 0.5 GB a day, 7 days (from 2026-12-01) | Well under; the watchdog logs one line per run |
| GitHub Actions heartbeat | Free on a public repo | 24 short runs a day |
| GitHub issues and GitHub Mobile | Free | A few incident issues a month |
| Cloudflare Notifications | Email and up to 100 webhooks on every plan | A handful |

No new vendor. Claude Code build effort is about 5 PRs over the four phases, inside
the $25 a month budget.

## Revisit

- If Cloudflare Monitors are free and cover error-rate alerts, move the
  error-spike rule there and keep the watchdog for HTTP checks.
- If a page is missed, or Jim wants pages that escalate to a phone call,
  put PagerDuty Free (or Pushover, $4.99 once) to him again; the watchdog
  only needs a second sender.
- If logs outgrow 0.5 GB a day, sample before paying.

## Consequences

- Every production site and Worker is checked every 5 minutes, and an
  outage reaches Jim's phone in about 10 minutes instead of at the next
  nightly run or weekly review, with no new vendor.
- A new deploy target can't ship unwatched: the audit fails until it has
  a check.
- The tech radar lists the watchdog and Cloudflare Monitors and
  Notifications in Assess until phase 1 ships, PagerDuty, ntfy.sh and
  Pushover in Assess as the pagers to revisit, and Grafana Cloud, Better
  Stack, UptimeRobot, Healthchecks.io, Opsgenie and the self-hosted
  monitors in Hold.
- Observability has one home, `observability/`, and one reviewer role,
  `observability-reviewer`. The review roles build (decision 0019's
  phases) adds the domain and the project's tier.
- GitHub issues come back for one job, incident alerts, which the
  watchdog opens and closes; work is still tracked in issue files.
- New secrets (the alerts App key and the webhook secret) get rows in
  `secret-rotation.md`, and the App gets a settings check in
  `standards/settings.toml`, in the phase that adds them.
