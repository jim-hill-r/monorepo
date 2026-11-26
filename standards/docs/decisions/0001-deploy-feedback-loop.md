# 0001. Preview builds, recorded smoke UAT and real-user review on free tiers

- Status: Accepted
- Date: 2026-11-26
- Plan: [../plans/2026-11-deploy-feedback.md](../plans/2026-11-deploy-feedback.md)

## Context

Jim wants better feedback on every change: a cleaner CD, a UAT step after each
deploy, observability, and above all a way to watch how real people use the
apps as part of the development workflow. It has to stay cheap.

The repo ships several kinds of deliverable, and each needs checking in its
own way. The full inventory is in the plan; the ones users touch are:

- **Websites:** Dioxus web apps (cookbook, pane, cahokia, sparkhill/webapp,
  blueeel, and bear_down soon) and Perseus sites (sparkhill/website,
  profiles/jimhillr).
- **Native apps:** cahokia's Dioxus desktop and mobile apps.
- **CLIs:** homer, cookbook's recipe_gen and the my_lang compiler.
- **Bevy games:** controlled_chaos (starcraft is on hold).
- **An HTTP API:** luggage (on hold).

Libraries, docs and content are checked by their consumers and by CI only.

The websites are served as static files from Cloudflare Pages or Workers.
Until the mise migration, CD only ran when CI committed build artifacts to main, so a
deploy left no trace on the PR that caused it and nobody looked at the result.
[PR #570](https://github.com/jimhillr/monorepo/pull/570) replaced that with
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
5. **CLIs and Bevy games get the same three checkpoints in their own form.**
   - Before merge: CLIs run scripted user sessions against the built binary
     with transcript tests (`trycmd`), and a PR that changes user-facing
     output gets a terminal recording (a `vhs` GIF). Bevy games build for the
     web (`wasm32`) where the game supports it, so the PR gets a playable
     preview like a website, and a scripted headless run under xvfb captures
     screenshots at fixed frames and a short video.
   - After release: CLIs publish binaries to a GitHub Release, and the smoke
     session runs again against the released binary. Games publish their web
     build to Cloudflare Pages and native builds to a GitHub Release.
   - Real use: games send a few gameplay events (session start, level
     reached, quit, crash) to PostHog from the web build. CLIs send nothing
     unless the user opts in with an environment variable, in which case
     errors and command timings go to Honeycomb through `tracing`.
6. **Native apps and the API are covered more lightly.** cahokia desktop gets
   a recorded launch under xvfb; cahokia mobile is checked through its shared
   `ui` crate in the web build, with an Android emulator run only nightly.
   iOS needs macOS runners and is not covered. luggage gets an HTTP smoke
   test against its OpenAPI routes and `tracing` export to Honeycomb once it
   is deployed.
7. **Playwright screenshots and video are the default evidence on PRs.** Jim
   asked for this on 2026-11-26. Every UI change links the recorded report in
   the PR, which replaces taking screenshots by hand.
8. **Every project is built, released or deployed, and UAT tested at least
   once a month, even when nothing changed** (Jim, 2026-11-26). A monthly
   scheduled workflow runs `ci`, `release`, `deploy` where a project deploys,
   and `smoke` for every project, and publishes the recorded UAT report.
   Toolchain drift, expired secrets and dead dependencies then show up within
   a month instead of when someone next touches the project. A project whose
   last successful UAT is older than 31 days is flagged.
9. **Everything stays inside free tiers** and runs from GitHub Actions or an
   agent skill, never from a local clone.

## Consequences

- Reviewers get a link, screenshots and a video (or a terminal recording) for
  every user-facing change, which
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
- Bevy games need a deterministic scripted mode (fixed timestep, seeded RNG,
  a list of inputs) before a recorded run means anything. Games that cannot
  build for `wasm32` get only the recorded native run.
- CLI telemetry is opt-in only, so most real-use feedback for CLIs comes from
  Jim's own runs and the transcript tests.
- The monthly run deploys projects whose code did not change, so a deploy
  must be safe to repeat, and an on-hold project still has to build. An
  on-hold project that cannot is recorded as a `FIX` in its `ISSUES.md`
  rather than skipped silently.
- The smoke suite must stay short (about a minute per app) to keep Actions
  minutes low, and must not depend on seeded data in production.
- New secrets: `HONEYCOMB_API_KEY` for CI and deploy markers, and a PostHog
  personal API key (read-only) for the weekly review task. The PostHog project
  key is public and ingest-only, so it lives in each app's config.
- Workflow changes build on the mise workflows from PR #570
  (`mise-projects.yml`, `push-main.yml`), not the Cast ones.
