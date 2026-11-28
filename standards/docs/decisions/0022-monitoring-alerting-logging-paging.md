# 0022. Monitoring, alerting, logging and paging: Cloudflare for signals and logs, a watchdog Worker for checks and alert rules, PagerDuty Free to page Jim's phone

- Status: Proposed
- Date: 2026-11-28
- Asked by: Jim
- Builds on: [0001](0001-deploy-feedback-loop.md) (Cloudflare is the
  observability backend, PostHog only for replay),
  [0003](0003-eval-engine.md) (production evals),
  [0007](0007-infrastructure-as-code.md) (IaC),
  [0010](0010-credential-second-approval.md) and
  [0011](0011-mint-secrets-by-workflow.md) (credentials)

## Context

Jim (2026-11-28):

> We need to do tech selection on monitoring, alerting, logging and paging
> solutions. Lets look for many options and decide on the best one.

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
4. **Paging:** reach Jim's phone with a push that he acknowledges, and
   escalate to a phone call or SMS if he doesn't.
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
| **Rules as code in the watchdog, deduplicated by the pager** | Each check has a stable key. The watchdog sends "trigger" when a check fails twice in a row and "resolve" when it passes, and the pager's dedup key turns that into one incident per outage, so the Worker keeps no alert state of its own. **Chosen.** |
| Cloudflare Notifications | Free for email and webhooks on every plan (up to 100 webhook destinations); PagerDuty as a destination needs Business. Pages "Deployment failed" is free. **Used as an input:** its webhook points at the watchdog, which forwards to the pager. |
| Grafana Alerting | Good, but only sees what is in Grafana. |
| GitHub Actions failure notifications | Already email and GitHub Mobile; kept for CI. Nightly CD and Infra apply failures also send a low-urgency alert. |

### Paging

| Option | Cost | Phone reach | Fit |
| --- | --- | --- | --- |
| **PagerDuty Free** | $0, 5 users, 1 escalation policy, 1 schedule | App push, email, and 100 SMS and phone calls a month | Real paging: acknowledge, escalate, re-notify, high and low urgency, Events API v2 with dedup keys, an official Terraform provider and MCP server, 750+ integrations. **Chosen.** |
| ntfy.sh | $0, no account | Android push can override Do Not Disturb if the user allows it; the iOS app calls itself "bare bones" | Simple, but best effort on one server, no acknowledge or escalation, and its terms say it must not be the "sole or primary means" of alerts for critical infrastructure. Topic names act as the password. |
| Pushover | $4.99 once per platform after a 30-day trial | Emergency priority repeats until acknowledged; high priority bypasses quiet hours | The best phone experience for one person, but it costs money and needs a card, so it breaks the free-tier rule without Jim's say-so. Second choice. |
| Grafana IRM (OnCall) | $0, 3 IRM users | Mobile app; phone and SMS on Free not confirmed | Fine if we were already on Grafana; we aren't. |
| Better Stack on-call | Phone and SMS need a $29 a month seat | | Paid. |
| Opsgenie | | | Atlassian has stopped selling it and is retiring it. |
| GitHub Mobile notifications (an alert opens a GitHub issue or comment that mentions Jim) | $0, approved vendor | Push only; no override of Do Not Disturb, no escalation | No new vendor, but a quiet phone misses it, and the repo tracks work in issue files, not GitHub issues. Kept as the fallback channel. |
| Email from a Cloudflare Email Worker, Telegram or Discord bots | $0 | Push at the app's normal level | No acknowledge or escalation. |

### Whole-stack bundles considered

| Bundle | Money | New vendors | Why not, or why |
| --- | --- | --- | --- |
| **A. Cloudflare + watchdog Worker + PagerDuty Free** | $0 | 1 (PagerDuty) | **Recommended.** Everything but the pager runs where we already are. |
| B. Cloudflare + watchdog Worker + ntfy.sh | $0 | 1 (ntfy) | No acknowledgement or escalation, and its terms rule out critical-alert use. |
| C. Cloudflare + watchdog Worker + Pushover | $4.99 once per phone platform | 1 (Pushover) | Best push, but needs a card and Jim raising a budget from $0. |
| D. GitHub only (Upptime + GitHub Mobile) | $0 | 0 | Late scheduled runs and a push a quiet phone hides; fails "reach Jim's phone". |
| E. Grafana Cloud Free for all four | $0 | 1 (Grafana) | Worker logs can't reach it without paid export, so logs stay split anyway; 3-user cap; a second rule store. |
| F. Better Stack Free | $0 for email only | 1 | No phone or SMS without a $29 a month seat. |

## Decision

Bundle A, in four parts.

1. **Logging stays on Cloudflare.** Workers log structured JSON through
   `tracing` (one line per request with `project`, `build`, route, status
   and duration, never a user's personal data), which Workers Logs indexes
   and Workers Observability queries for 7 days. Browsers keep sending
   errors to `telemetry`. Anything worth keeping past a week becomes an
   issue file or a committed export, as 0001 already says. No log shipping
   to another vendor.
2. **Monitoring is a `watchdog` project:** a Rust Worker on one Cron
   Trigger that runs every 5 minutes. It reads its checks from
   `watchdog/checks.toml`, which an audit check keeps in step with every
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
   - **High urgency (pages Jim's phone):** a production site or Worker
     failing two checks in a row (about 10 minutes down), a fivefold error
     rise after a deploy, the watchdog silent for an hour, and a free-tier
     limit that will stop a service today (for example Workers requests
     past 80% before 00:00 UTC).
   - **Low urgency (push only, no call, nothing at night):** nightly CD,
     Infra apply or production eval failures, Cloudflare Notifications
     such as Pages "Deployment failed", and free-tier usage past its
     warning level.
   - Each alert has a stable dedup key (`<project>/<check>`), triggers on
     failure and resolves on recovery, so one outage is one incident.
     Alert text names the project, check, URL and status, and never
     includes user data, request bodies or secrets.
   - Cloudflare Notifications that are free (email, webhooks) are turned
     on through IaC and point their webhook at the watchdog's `/notify`
     route, which checks the shared webhook secret and forwards.
4. **Paging is PagerDuty Free.** One service ("jimhillr production") with
   an Events API v2 integration, one escalation policy and one schedule,
   both just Jim. Jim's notification rules: high urgency is an app push at
   once, then a phone call after 5 minutes unacknowledged, then SMS;
   low urgency is a push only. GitHub Mobile and email stay as the
   fallback channel if PagerDuty is down.

### Setup by IaC and scripts

- The watchdog deploys like `telemetry` (`mise run deploy`,
  `free-tiers.sh worker`, the shared `cloudflare-worker.toml` tasks); its
  Cron Trigger counts against the FT-004 budget (1 of 3).
- The `infra` project gains a `pagerduty` stack for the service,
  escalation policy, schedule and Cloudflare notification policies. This
  needs terrars bindings for the `pagerduty/pagerduty` provider and for
  `cloudflare_notification_policy` and its webhooks, generated with the
  pinned terrars version.
- **No secret in state.** The Events API routing key and the webhook
  secret would be computed attributes in OpenTofu state, which IAC-002
  forbids. So, as in 0011, Infra Mint creates the Events API integration
  through PagerDuty's REST API and pipes the routing key straight into
  the watchdog's Worker secret and a GitHub secret, and rotates it each
  quarter. The PagerDuty admin API key is a bootstrap credential in the
  `infra-credentials` environment, and every change to it needs Jim's
  second approval (0010).
- Jim's phone number and notification rules are set by Jim in the
  PagerDuty app, never in git or state: a human task with a runbook.

## Threat model

| Threat | Mitigation |
| --- | --- |
| A leaked routing key lets someone spam Jim's phone | Only in a Worker secret and a GitHub secret; rotated quarterly by Infra Mint; PagerDuty rate-limits events; revoking it is one Infra Mint run. |
| A forged Cloudflare webhook to `/notify` | Checked against the webhook secret Cloudflare sends; the route only forwards known notification types; rate-limited like `telemetry`. |
| Alert text leaks user data to a new vendor | Alerts carry only project, check, URL, status and counts; a unit test rejects other fields. |
| The pager is down or the plan changes | GitHub Mobile and email fallback; PagerDuty has no card on file, so a plan change can't bill. Pushover is the documented second choice. |
| The watchdog dies silently | The hourly Actions heartbeat check pages. |
| Alert fatigue makes Jim mute it | Only outages and fast-moving free-tier stops page; everything else is low urgency; each page's rule is reviewed in the weekly review. |

## Phases

1. **Watchdog Worker** with HTTP checks, `checks.toml`, the audit check
   against `DEPLOY_URL`s, results in Analytics Engine, alerts to the log
   only. Find out whether Cloudflare Monitors are on the Free plan.
2. **PagerDuty:** Jim's human task to sign up for the free plan and
   install the app (with a runbook), the bootstrap API key in
   `infra-credentials`, the `pagerduty` IaC stack, Infra Mint for the
   routing key, then the watchdog sends trigger and resolve events. A
   test page on a fire drill proves the path to Jim's phone.
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
| PagerDuty Free | 5 users, 100 SMS and phone calls a month | 1 user; calls only after 5 unacknowledged minutes on high urgency, so a few a month |
| Cloudflare Notifications | Email and up to 100 webhooks on every plan | A handful |

One new vendor, PagerDuty, needs Jim's sign-off under the vendor rule.
Claude Code build effort is about 5 or 6 PRs over the four phases, inside
the $25 a month budget.

## Revisit

- If Cloudflare Monitors are free and cover error-rate alerts, move the
  error-spike rule there and keep the watchdog for HTTP checks.
- If PagerDuty's free plan drops phone calls or the 100 a month cap is
  hit, ask Jim about Pushover ($4.99 once).
- If logs outgrow 0.5 GB a day, sample before paying.

## Consequences

- Every production site and Worker is checked every 5 minutes, and an
  outage reaches Jim's phone in about 10 minutes instead of at the next
  nightly run or weekly review.
- A new deploy target can't ship unwatched: the audit fails until it has
  a check.
- The tech radar lists PagerDuty in Assess with this record as its source
  (Trial once Jim accepts it), ntfy.sh and Pushover in Assess as the
  fallback pagers, and Grafana Cloud, Better Stack, UptimeRobot,
  Healthchecks.io, Opsgenie and the self-hosted monitors in Hold.
- PagerDuty becomes a vendor with a billing check: a read-only API check
  in `billing-check.sh` if its API exposes the plan, otherwise a settings
  check for Jim in `standards/settings.toml`.
- New secrets (the PagerDuty admin API key, the routing key, the webhook
  secret) get rows in `secret-rotation.md` in the phase that adds them.
