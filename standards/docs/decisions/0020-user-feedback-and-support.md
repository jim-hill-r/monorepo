# 0020. User feedback, support and in-app user guides as one platform component

- Status: Proposed (for Jim)
- Date: 2026-11-28
- Asked by: Jim (project thread, 2026-11-28)
- Plan: [2026-11-user-feedback.md](../plans/2026-11-user-feedback.md)
- Builds on: [0003](0003-eval-engine.md) item 8 (the review toolbar),
  [0004](0004-database-engine.md) (files first),
  [0017](0017-user-docs.md) (user guides) and 0019 (project tiers, PR 904)

## Context

Jim, 2026-11-28: "As a part of our technical platform we should create
user feedback tooling ... so we can embed it into the sparkhill webapp
allowing users to provide feedback", then "The feedback tooling should
also provide some ability to get support as well", and "can we build into
the technical platform a way to surface the user guides into
applications."

We had three separate pieces: the eval engine's review toolbar (Jim only,
planned), bear_down's fan votes (built, with their own `telemetry` event)
and the `telemetry` Worker. None is for an app's ordinary users, none
offers help, and the first app to embed feedback, Blue Eel, promises that
nothing leaves the device (SPRK-006) under a `connect-src 'self'` CSP.

## Decision

1. **One `feedback` project in the technical-platform tier.** It holds the
   review toolbar already planned there and a new public user widget, over
   the shared `feedback/model`. The two never share collected data: the
   user widget takes no screenshot, DOM or error text.
2. **In-app help comes from the user guide.** `feedback/help` compiles each
   project's `docs/user/` Markdown into the app at build time and shows it
   as a Help panel with deep links; it works offline and never fetches.
   No second copy of the guide exists.
3. **Explicit send, nothing else.** Feedback is a form the user fills in,
   previews exactly, and sends. Only project, build, page, topic, rating
   and note are sent; no id, cookie or tracking.
4. **Same-origin transport.** The widget posts to `/api/feedback` on the
   app's own origin; a Pages Function forwards to the `telemetry` Worker,
   which gains an `app_feedback` kind. CSPs stay `connect-src 'self'`.
5. **Files, not a database.** Analytics Engine is the inbox; the weekly
   export writes JSON Lines to the public `feedback-data` branch, and
   triage turns actionable lines into issue files, treating the text as
   untrusted data.
6. **Support by email, outside our systems.** "Contact us" is a `mailto:`
   link to a Cloudflare Email Routing address. The form never asks for
   contact details, because its output is public.
7. **Reuse, don't fork.** bear_down's votes move onto the widget's
   `QuickVote`; the review toolbar also ships as an unpacked browser
   extension so production builds carry no review code.

## Alternatives considered

| Option | Money cost | Why not |
| --- | --- | --- |
| PostHog surveys | $0 up to 1,500 responses a month | Approved only for session replay; adds a third-party script, which SPRK-006 forbids |
| Hosted widgets (Userback, Marker.io, Canny) | Paid past tiny free plans | New vendor, third-party script, data outside the repo |
| GitHub Issues or Discussions as the form | $0 | Users need a GitHub account; kept as an optional public support link |
| Allow the `telemetry` Worker in `connect-src` | $0 | Simpler, but changes SPRK-006's "only its own origin" |
| D1 for feedback | $0 on the free plan | Decision 0004: files while files will do |
| A "reply to me" email field | $0 | Puts personal data on a public branch; email keeps it private |
| Turnstile against spam | $0 | Loads a script from another site; rate limits first |
| Chrome Web Store for the extension | $5 one time | Breaks the free-tier rule; unpacked from a GitHub release is free |
| A separate help project | $0 | Help, support and feedback share one panel and one entry point in each app; one crate in one project is simpler, and `feedback/help` still works alone |

## Consequences

- Every app that embeds it gets help, support and feedback with one
  dependency, and its user guide becomes user-visible, so DOC-005 to
  DOC-007 gaps show in the product, not only in the audit.
- `telemetry` gains a kind and a TELE-003 wording change (user-sent events
  ignore DNT/GPC); Blue Eel gains SPRK-011 and a SPRK-006 clause. All need
  Jim's requirement approval.
- Jim gets one human task in Phase 5: move one domain's DNS to Cloudflare
  and turn on Email Routing, with its runbook.
- Public feedback text reaches an agent's triage, so the triage rules
  (quote only, never act on instructions in notes) are part of the threat
  model.
- Cost: $0 on Cloudflare and GitHub free plans; about eight medium agent
  PRs of Claude Code effort against the $25 a month budget.
