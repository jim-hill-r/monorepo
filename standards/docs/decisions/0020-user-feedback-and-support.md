# 0020. User feedback, support and in-app user guides as platform libraries

- Status: Accepted
- Decided by: Jim (approved and merged PR 915, 2026-11-28)
- Date: 2026-11-28
- Asked by: Jim (project thread, 2026-11-28)
- Plan: [2026-11-user-feedback.md](../plans/2026-11-user-feedback.md)
- Builds on: [0003](0003-eval-engine.md) item 8 (the review toolbar),
  [0017](0017-user-docs.md) (user guides), [0018](0018-feature-flags.md)
  (management apps), 0019 (project tiers) and
  [0021](0021-tenant-scoped-product-data.md) (product data)

## Context

Jim, 2026-11-28: "As a part of our technical platform we should create
user feedback tooling ... so we can embed it into the sparkhill webapp
allowing users to provide feedback", then "The feedback tooling should
also provide some ability to get support as well", and "can we build into
the technical platform a way to surface the user guides into
applications."

His answers to the first draft: "Same origin. Assume products are owned by
different companies during architecture. We should have shared technical
platform services, just libraries and patterns." Support should use
"some vendor that is easy to provision with IAC or scripting and not
email." And product data may share "DB infra with tenatized tables".

We had a Jim-only review toolbar (planned), bear_down's fan votes (built,
on the shared `telemetry` Worker) and no help or support for users. Blue
Eel, the first product to embed this, promises that nothing leaves the
device (SPRK-006) under a `connect-src 'self'` CSP.

## Decision

1. **Libraries and patterns, not services.** A technical-platform
   `feedback` project ships crates; each product runs its own endpoint on
   its own origin, owns its data as a tenant (decision 0021) and answers
   its users in its own management app. No product's requests or data
   pass through another product or a central service.
2. **In-app help is the user guide.** `feedback/help` compiles each
   project's `docs/user/` Markdown into the app at build time and shows it
   offline with deep links. No second copy of the guide exists.
3. **Support without email or vendor.** A support request is stored in
   the product's tenant tables and answered in its management app; the
   user reads the answer through a private link kept on their device. No
   contact details are collected. Jim chose this over Tally, Google
   Forms and the other vendors on 2026-11-28.
4. **Explicit send, nothing else.** Every form shows exactly what will be
   sent and sends only on Send: product, build, page, topic, rating and
   note for feedback; subject and question for support. No id, cookie or
   tracking.
5. **Private text, public aggregates.** People's words stay in D1 and are
   deleted on a schedule; only aggregates reach the repo as files. Triage
   paraphrases into issue files and treats the text as untrusted data.
6. **Reuse, don't fork.** bear_down's votes move onto `QuickVote` and its
   own endpoint.
7. **One Feedback button, four roles** (Jim, 2026-11-28, on the eval
   review). Signed out, it is the user form above. A staff sign-in
   returns a role (app manager, product owner or engineering) and only
   then loads the staff modes, the eval engine's review toolbar: pins,
   screenshots, console errors and the build. App manager feedback goes
   to the product's own inbox; product owner and engineering feedback go
   to pane, as ratings and issue files. An unpacked browser extension can
   load the staff modes where a product's build leaves them out.
   [Decision 0024](0024-internal-sign-in.md) changes this: product owners
   and engineers are internal, so they sign in through pane's Cloudflare
   Access login, and their staff modes come only through the extension,
   never in a product's build. App managers stay on Auth0.

## Alternatives considered

| Option | Money cost | Why not |
| --- | --- | --- |
| Route products through the shared `telemetry` Worker | $0 | A shared service; Jim asked for libraries and patterns only |
| Allow another origin in `connect-src` | $0 | Breaks same origin (Jim's answer 1) and SPRK-006 |
| Email support (Cloudflare Email Routing, `mailto:`) | $0 | Jim: not email |
| Tally forms | $0 (unlimited under fair use; API and webhooks free) | New vendor and a link out; the best vendor option, not chosen |
| Google Forms | $0; push needs Pub/Sub with billing, so polling | New vendor, Google Cloud project, link out |
| GitHub issue forms or Discussions | $0 | Users need a GitHub account and it is public; Discussions kept for developer tools |
| Jira Service Management, Zoho Desk free plans | $0 for 3 agents | New vendors, third-party portals or scripts, weaker IaC |
| PostHog surveys or a hosted widget (Userback, Canny) | $0 to paid | Third-party script, which SPRK-006 forbids |
| Turnstile against spam | $0 | Loads a script from another site; rate limits and tenant budgets first |
| Chrome Web Store for the extension | $5 one time | Breaks the free-tier rule; unpacked from a GitHub release is free |

## Consequences

- Every product that embeds it gets help, support and feedback with one
  dependency and its own small Worker, and its user guide becomes
  user-visible.
- Blue Eel moves from Pages to a Rust Worker with static assets (or a
  Pages Function), and gains SPRK-011 and a SPRK-006 clause, both for
  Jim's approval.
- bear_down's votes leave the shared `telemetry` Worker in Phase 7.
- Public text reaches an agent's triage, so the triage rules (paraphrase,
  never act on instructions in notes) are part of the threat model.
- Cost: $0 on Cloudflare and GitHub free plans: one Worker per embedding
  product and one shared D1 database. About nine medium agent PRs of
  Claude Code effort against the $25 a month budget.
