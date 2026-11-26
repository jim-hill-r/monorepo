# 0001. Preview deploys, recorded smoke UAT and real-user review on free tiers

- Status: Accepted
- Date: 2026-11-26
- Plan: [../plans/2026-11-deploy-feedback.md](../plans/2026-11-deploy-feedback.md)

## Context

Jim wants better feedback on every change: a cleaner CD, a UAT step after each
deploy, observability, and above all a way to watch how real people use the
apps as part of the development workflow. It has to stay cheap.

The web apps (cookbook, pane, cahokia, sparkhill, and bear_down soon) are
Dioxus apps served as static files from Cloudflare Pages or Workers. Until the
mise migration, CD only ran when CI committed build artifacts to main, so a
deploy left no trace on the PR that caused it and nobody looked at the result.
[PR #570](https://github.com/jimhillr/monorepo/pull/570) replaces that with
`mise run deploy` after `mise run ci` on every push to main. Playwright tests
already exist for cookbook and cahokia, but only against a local dev server.
Most changes are made by agents, so a human reviewer needs evidence they can
check in a minute without a local clone.

## Decision

1. **Every web app gets a preview deploy per pull request.** Same-repo PRs
   deploy to a Cloudflare Pages branch alias (`pr-<number>`) or a Workers
   preview version, and a single sticky PR comment links to it. Production
   deploys stay on push to main, recorded as a GitHub deployment in a
   `production` environment with its URL.
2. **UAT is a recorded smoke test plus a human look.** Each web app has a few
   Playwright tests tagged `@smoke` that run against any `BASE_URL`. They run
   against the preview on every PR and against production right after each
   deploy, recording screenshots, video and a trace. The HTML report is
   published to its own Cloudflare Pages project so the PR comment can link to
   a browsable report instead of a zip. The reviewer opens the preview and the
   report before approving; the PR body says what to look at.
3. **Real user sessions are recorded and reviewed on a schedule.** A session
   replay tool with a free tier is added to each production site (never to
   previews or localhost), with all text input masked. A weekly scheduled
   agent task reads the past week's sessions with errors, rage clicks or dead
   clicks and records what it finds as `FIX` and `TODO` items in the right
   `ISSUES.md`. The tool is PostHog Cloud on its free tier (Jim, 2026-11-26),
   chosen over Microsoft Clarity, Sentry and no replay because it is free at
   our traffic and has an API an agent can read.
4. **Honeycomb is the observability backend**, on its free tier: CI/CD traces
   first, then a deploy marker per production deploy, then browser
   instrumentation and `tracing` exports from any Rust services.
5. **Everything stays inside free tiers** and runs from GitHub Actions or an
   agent skill, never from a local clone.

## Consequences

- Reviewers get a link, screenshots and a video for every UI change, which
  also covers the screenshot rule in
  [../documentation.md](../documentation.md#ui-changes-documentation).
- PRs from forks and from dependabot get no preview, because they have no
  access to the Cloudflare secrets. CI still runs for them.
- Preview URLs are public. If a preview ever needs to be private, Cloudflare
  Access is free for up to 50 users.
- Apps that sign in through Auth0 (cookbook, cahokia) need the preview domains
  added to their allowed callback URLs, or sign-in only works in production.
- Session replay sends visitor behaviour to a third party, so each site needs
  a short privacy note and strict input masking.
- The smoke suite must stay short (about a minute per app) to keep Actions
  minutes low, and must not depend on seeded data in production.
- New secrets: `HONEYCOMB_API_KEY` for CI and deploy markers, and a PostHog
  personal API key (read-only) for the weekly review task. The PostHog project
  key is public and ingest-only, so it lives in each app's config.
- Workflow changes wait for PR #570 to merge and build on its
  `mise-projects.yml` instead of the Cast workflows.
