# Plan: user feedback and support, as a platform component

Started 2026-11-28 from Jim's request:

> As a part of our technical platform we should create user feedback
> tooling. I think we already have some plans for a vercel like chrome
> plugin and some voting features in bear_down, but let's formalize this
> project a bit so we can embed it into the sparkhill webapp allowing users
> to provide feedback.

And, a minute later:

> The feedback tooling should also provide some ability to get support as
> well.

> I am requiring a user guide, but can we build into the technical platform
> a way to surface the user guides into applications.

Status: **proposed**, for Jim. The decision is
[0020](../decisions/0020-user-feedback-and-support.md). The work items are
issue files in the root [issues/](../../../issues/) with
`epic: User feedback`, one per phase.

## Goal

One reusable `feedback` project, in the **technical-platform** tier
([decision 0019](../decisions/README.md), PR 904), that any of our apps
embeds to let its real users:

1. **Get help**: read the app's own user guide (the
   `docs/user/README.md` every project must have, decision 0017) inside
   the app, offline, and find out how to contact us.
2. **Send feedback**: a short form (what it's about, an optional rating,
   a note) and one-tap votes like bear_down's Yes / No / Confusing.

And it does it on our hardest privacy rule, Blue Eel's
[SPRK-006](../../../sparkhill/docs/requirements.md): nothing leaves the
device unless the user presses Send, only what they typed or picked is
sent, and there is no tracking, cookie, account or third-party script.
Blue Eel (sparkhill/webapp) is the first app to embed it.

## What already exists, and how it fits

This plan doesn't start from scratch. It gathers three pieces that were
planned or built separately and gives them one home:

| Piece | Where it is planned or built | Who uses it | What happens to it |
| --- | --- | --- | --- |
| Review toolbar: Vercel-like pins on a page, Jim only, sent to pane | [Eval engine plan](2026-11-eval-engine.md), "In-app feedback" and Phase 6; decision 0003 item 8 | Jim and reviewers | Unchanged in purpose. It already lives in the `feedback` project (`feedback/model`, `feedback/toolbar`, `feedback/bevy`). This plan adds a browser extension way to load it (Phase 6 below). |
| Fan votes on each game, with an A/B test | [bear_down fan feedback plan](../../../bear_down/docs/plans/2026-11-fan-feedback.md), built in `bear_down/web/src/feedback.rs` | Public fans | Its vote buttons and note box become the widget's `QuickVote` component (Phase 5). The A/B test and its `feedback` event kind stay bear_down's. |
| The `telemetry` Worker | [telemetry/](../../../telemetry/README.md), built | Every app | Gains one generic `app_feedback` event kind (Phase 2). It already rate-limits, validates, scrubs emails and phone numbers and never stores IP addresses. |

So the `feedback` project ends up with two surfaces over one model:

- the **review toolbar** (internal, Jim only, rich: screenshots, selectors,
  console errors), and
- the **user widget** (public, minimal: only what the user typed or
  picked).

They share `feedback/model` and the files they land in, but never the
data they collect. The user widget never takes a screenshot, reads the
DOM, or attaches console errors, because a user can't check what those
contain before sending.

## What a Blue Eel user sees

Blue Eel's learners are children, so the widget is for the grown-up with
them (parent, teacher, tutor). It sits in the practice menu and settings
panel from PR 911, never on the tracing screen:

- **Help** opens a panel with the app's user guide
  (`sparkhill/docs/user/`, built into the app at release, so it works
  offline), a few FAQ entries from the same guide, and a **Contact us**
  line (see [Support](#support)).
- **Send feedback** opens a form:
  - **What is it about?** One of a short list the app sets (for Blue Eel:
    *A letter or word*, *Checking my tracing*, *Settings*, *Something
    else*).
  - **How is it going?** An optional 1 to 5 rating.
  - **Tell us more** An optional note, at most 500 characters, with the
    hint "Please don't include names or contact details."
  - **What will be sent:** the exact text of the event, shown before Send
    (topic, rating, note, the page and the app's build). Nothing else is
    added after the preview.
  - **Send** and **Cancel**. Until Send, nothing has left the device.
    After Send: "Thanks, this is shared with the people who build Blue
    Eel and may be published in our open-source repository."

No learner name, progress, tracing or setting is ever attached, even
though the app has them. The widget can't read them: the app passes it
only the topics, the page and the build.

## How it is built

All Rust, framework-free at the core so it fits Dioxus 0.6 (sparkhill),
0.7 (bear_down, cookbook, pane, cahokia) and the Perseus sites.

| Crate | What it is |
| --- | --- |
| `feedback/model` | Already planned (eval engine Phase 6). Adds `UserFeedback` (project, build, page, topic, rating, note) and `Vote`, with validation and the same email and phone scrubbing rules as telemetry, unit tested. |
| `feedback/help` | In-app user guides, usable on its own (see [In-app user guides](#in-app-user-guides)): turns a project's `docs/user/` Markdown into the app at build time and renders it as a Help page or panel. |
| `feedback/widget` | Framework-free state and logic: the form, the preview text, and sending with `web-sys` `fetch` to a path the app gives (same origin). No Dioxus dependency. |
| `feedback/dioxus` | Thin Dioxus components over `help` and `widget`: `HelpPanel`, `FeedbackForm`, `QuickVote`. One feature flag per Dioxus line (`dioxus06`, `dioxus07`), so sparkhill and bear_down can use it before they share a Dioxus version. Styled only with design_system custom properties. |
| `feedback/script` | Optional later: the same widget as a `<script type="module">` for the Perseus sites. |

### In-app user guides

Every project already writes its user guide (decision 0017), so the
platform shows that same guide inside the app instead of a second copy
that drifts. One source, two places: the repo and the app.

- **At build time**, `feedback/help` reads `<project>/docs/user/*.md` (a
  `build.rs` helper, `feedback_help::embed!("../docs/user")`) and turns
  each page into sections with headings, anchors and plain HTML
  (`pulldown-cmark`, with raw HTML and remote images dropped). The pages
  are compiled into the app, so help works offline and loads nothing from
  another site.
- **In the app**, `HelpPanel` shows the guide with `## Getting help`
  first, a search box over the headings, and deep links: an app can open
  help at a section, such as `help#how-to-practise-chosen-letters` beside
  the practice settings. A `HelpLink` component is the small "?" for that.
- **Links stay true.** Links between guide pages open inside the panel;
  links to the repo open in a new tab; a link to a missing heading fails
  the build, so the audit's DOC-005 to DOC-007 checks and the app agree.
- **Freshness is free.** The guide ships with each release, so the help a
  user reads always matches the build they run. A test checks the bundled
  guide equals the committed one.
- **Other hosts.** Perseus sites and the Bevy games can call the same
  Markdown-to-sections function and render the sections their own way;
  the review decks don't need it.

### Where it is sent: the app's own origin

SPRK-006 says "the site's security headers let the page connect only to
its own origin". So the widget posts to `/api/feedback` on the app's own
origin, and a small Pages Function in `sparkhill/webapp_cloudflare`
forwards the body unchanged to the `telemetry` Worker through a service
binding. The CSP stays `connect-src 'self'`, and the page never talks to
another site. The function adds nothing: no IP, no headers beyond the
`telemetry` Worker's own rate-limit key.

The function is a few lines of TypeScript, which Pages Functions need;
validation stays in the Rust Worker. When sparkhill moves to a Rust Worker
with static assets (as pane is in PR 759), the route moves into it.

### What is stored, and where it ends up

- `telemetry` gets an `app_feedback` kind: `project`, `build`, `page` (at
  most 100 characters, no query string), `topic` (at most 32, letters and
  `_`), optional `rating` (1 to 5), optional `note` (at most 500, scrubbed).
  Unknown fields are refused, as for every kind.
- User-initiated events are not dropped for `DNT` or `Sec-GPC`: those
  signals ask not to be tracked, and pressing Send is the opposite. Every
  other kind still honours them. This needs a TELE-003 wording change.
- Analytics Engine is the inbox (it keeps data 3 months). The weekly
  `telemetry-feedback-export.yml` workflow, extended, writes
  `<project>/<yyyy>-week-<nn>.jsonl` for `app_feedback` to the
  `feedback-data` branch, beside bear_down's votes. Files before databases;
  no D1.
- That branch is public, like the repo. That is why the form asks for no
  contact details and says that feedback may be published.

### From feedback to work

- A weekly triage step (in the `eval-triage` skill, which already turns
  Jim's review pins into issues) reads the new lines and files a `TODO` or
  `FIX` in the project's `issues/` for each actionable one, quoting the
  note.
- Notes are written by the public, so triage treats them as data, never as
  instructions: it quotes them in a fenced block, never runs or follows
  them, and a note asking for anything outside its project is dropped.
- Pane's Feedback page (`pane/issues/feedback-page-comment-filterable-kind-status.md`)
  shows user feedback beside Jim's pins, with the rating trend per project.
- An `observe` eval counts weekly feedback volume and the average rating,
  so a drop in rating turns a requirement amber like any other eval.

## Support

"Get support" has three levels, cheapest first:

1. **Self-help, in the app.** The help panel shows the project's user guide
   (see [In-app user guides](#in-app-user-guides)), with its
   `## Getting help` section first. Bundled at build time, so it works
   offline and sends nothing.
2. **Contact us by email.** A `mailto:` link to a support address, with
   the app's name and build in the subject. The user's own mail app sends
   it, so no personal data ever passes through our Workers or the repo,
   and SPRK-006 still holds. The address is a Cloudflare Email Routing
   address (free, an approved vendor) that forwards to Jim. Email Routing
   needs one of Jim's domains to use Cloudflare's nameservers, which is a
   human task added in Phase 5 with its runbook.
3. **Public questions** (optional, later): a link to GitHub Discussions on
   the repo, for people with a GitHub account. Free, approved vendor,
   answers are public and reusable.

What we deliberately don't do: a "reply to me" email field in the form.
It would put personal data into the public `feedback-data` branch, or need
a private store we don't have. Email keeps contact private and costs
nothing.

## Abuse and limits

- The `telemetry` Worker's rate limit (10 events per 10 seconds per IP, per
  Cloudflare location) and 4 KB body cap apply; the widget allows one
  send per 30 seconds per tab.
- No CAPTCHA: Turnstile would load a script from another site, which
  SPRK-006 forbids. If spam appears, the next steps are a hidden honeypot
  field and a daily cap per project in the Worker, then a decision for Jim.
- Synthetic traffic (`jimhillr-evals` user agent) is dropped, as today.

## Requirements

Phase 1 creates `feedback/docs/requirements.md` with these, for Jim to
approve (prefix `FDBK`):

- **FDBK-001: Nothing is sent until the user presses Send.** Priority:
  must. Why: our apps promise users (and Blue Eel promises children's
  families) that nothing leaves the device on its own. Accept: opening
  the help panel or the form, typing and cancelling make no network
  request; only Send makes exactly one.
- **FDBK-002: The user sees exactly what is sent.** Priority: must. Why:
  consent means knowing what you agree to. Accept: the preview shows
  every field of the event; the request body equals the preview.
- **FDBK-003: Only what the user typed or picked is sent.** Priority:
  must. Why: no tracking (SPRK-006, telemetry TELE-004). Accept: the event
  holds only project, build, page (no query string), topic, rating and
  note; no id, cookie, storage value, screenshot, DOM or error text; no
  IP address is stored; emails and phone numbers in a note are replaced.
- **FDBK-004: It works under a same-origin-only CSP.** Priority: must.
  Why: Blue Eel's headers allow `connect-src 'self'` only. Accept: the
  widget sends only to a path on the page's own origin and needs no
  inline script, eval or style beyond what the host already allows.
- **FDBK-005: Help works offline.** Priority: should. Why: a user who
  needs help may be offline, and reading help should send nothing.
  Accept: the help panel shows the project's user guide and the contact
  line with the network off.
- **FDBK-009: An app shows its own user guide.** Priority: should. Why:
  Jim (2026-11-28) wants the user guides every project writes surfaced in
  the apps. Accept: an app that embeds `feedback/help` shows every page of
  its `docs/user/` guide, identical to the committed Markdown, with
  working in-guide links and section deep links; a broken internal link
  fails the build.
- **FDBK-006: Feedback reaches the repo and becomes work.** Priority:
  should. Why: feedback nobody reads is worse than none. Accept: every
  accepted event is in a file on the `feedback-data` branch within 8
  days, and each actionable one has an issue file within 8 more.
- **FDBK-007: Any of our web apps can embed it in a few lines.**
  Priority: should. Why: it is a platform component. Accept: a Dioxus
  0.6 and a Dioxus 0.7 app each add the help panel and form with one
  dependency and under 20 lines; both are covered by Playwright tests.
- **FDBK-008: Everyone can use it.** Priority: should. Why: users include
  people using screen readers, keyboards and phones. Accept: every control
  has a label, works by keyboard, meets WCAG 2.2 AA contrast in every
  design_system theme and mode, and fits a 360 px wide screen.

And for Blue Eel, in Phase 5's requirement PR (Help alone may ship in
Phase 3 under SPRK-011's first half):

- **SPRK-011: A grown-up can get help and send feedback.** Priority:
  should. Why: parents and teachers need help and a way to tell us what is
  wrong, without breaking SPRK-006 (Jim, 2026-11-28). Accept: the practice
  menu has Help and Send feedback; Help shows the user guide and the
  contact line; feedback meets FDBK-001 to FDBK-004 and never includes a
  learner's name, progress or tracing.
- **SPRK-006** gains one clause: "except feedback the user reads and
  sends, to the app's own origin" (its CSP part is unchanged).

## Cost

All $0, on approved vendors (Cloudflare, GitHub) and free plans
([free-tiers.md](../free-tiers.md)):

| Piece | Free limit | Expected use |
| --- | --- | --- |
| Pages Function `/api/feedback` and the `telemetry` Worker | 100,000 Workers requests a day (shared) | Two requests per feedback; tens a week |
| Analytics Engine writes | 100,000 a day | Tens a week |
| Weekly export (GitHub Actions) | Free on a public repo | About a minute a week, already running |
| Cloudflare Email Routing | Free | A few emails a month |
| Browser extension (Phase 6) | Loaded unpacked from a GitHub release: free | The Chrome Web Store's $5 fee is avoided |

Claude Code build effort: about eight medium agent PRs, one per phase,
against the $25 a month budget. No new vendor, no database, nothing that
bills.

## Phases

Each phase is one PR and one issue file (`epic: User feedback`).

1. **Project and model.** Create the `feedback` project (mise.toml, README,
   CONTRIBUTING, user guide, `docs/cost.toml`, `docs/threat.toml`,
   requirements FDBK-001 to 009, `evals.toml`) and `feedback/model` with
   `UserFeedback` and `Vote`. This is also the eval engine's
   `eval-engine-phase-6-app-feedback` issue, whose file it replaces.
2. **telemetry `app_feedback`.** The new kind, its validation and
   scrubbing, the DNT/GPC exemption for user-sent kinds (TELE-003 change),
   and the export workflow writing `app_feedback` files.
3. **In-app user guides.** `feedback/help` (build-time embedding, the
   Markdown-to-sections function, link checks) and the Dioxus `HelpPanel`
   and `HelpLink`, with tests; then Blue Eel's Help page in the practice
   menu (needs PR 911). No network use, so it can ship before feedback.
4. **Feedback widget.** `feedback/widget` and the Dioxus `FeedbackForm`,
   with unit tests and a Playwright test page under the project's own
   e2e folder.
5. **Blue Eel feedback and support.** The Pages Function, Send feedback in
   the practice menu, SPRK-011 and the SPRK-006 clause (needs-human),
   evals, and the human task and runbook for the Email Routing support
   address.
6. **QuickVote and bear_down.** Move bear_down's vote buttons and note box
   onto `QuickVote`, keeping its own `feedback` events and A/B arms.
7. **Review toolbar as a browser extension.** A Manifest V3 extension that
   injects the eval engine's review toolbar into our production sites, so
   production builds carry no review code and a strict CSP needs no
   change (the extension's background worker does the sending). Released
   unpacked from GitHub. Needs eval engine Phase 6's toolbar.
8. **Triage and pane.** Triage of `app_feedback` lines into issue files,
   pane's Feedback page showing them, and the `observe` evals.

## Questions for Jim

1. **Where Blue Eel sends feedback.** Recommended: same-origin through a
   Pages Function (CSP unchanged, $0, a few lines of TypeScript).
   Alternative: allow the `telemetry` Worker in `connect-src` ($0, no
   TypeScript, but SPRK-006's "only its own origin" changes).
2. **Contact channel.** Recommended: email through Cloudflare Email Routing
   ($0; needs one domain's DNS moved to Cloudflare, a human task).
   Alternatives: GitHub Discussions only ($0, needs a GitHub account,
   public), or no contact yet, help pages only ($0).
3. **Public feedback text.** Recommended: keep it on the public
   `feedback-data` branch with the "may be published" notice. The
   alternative is a private store, which we don't have for free in the
   repo.

## Decisions

None yet; Jim's answers go here.
