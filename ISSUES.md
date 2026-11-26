
# Priority Issues

# Backlog

Feedback on every change: see
[standards/docs/plans/2026-11-deploy-feedback.md](standards/docs/plans/2026-11-deploy-feedback.md).
Start these only after PR #570 (mise CI/CD) merges, because they change the
workflows it replaces.

- TODO (agent-generated): CD cleanup. Give each deploying project a `production` GitHub environment with its URL, write the deployed URL to the job summary, record the URL as `DEPLOY_URL` in the project's `mise.toml`, and add a `workflow_dispatch` redeploy of one project at a given ref for rollbacks.
- TODO (agent-generated): Preview deploys. Add a `preview` task to `standards/tasks/cloudflare-pages.toml` (`wrangler pages deploy --branch pr-<n>`), run it in PR CI for affected deploy projects on same-repo PRs, and keep one sticky PR comment with the preview URLs.
- TODO (agent-ignore): Add the Cloudflare Pages preview domains to the Auth0 allowed callback URLs for cookbook and cahokia. Needs Jim in the Auth0 dashboard.
- TODO (agent-generated): Smoke UAT. Tag a few Playwright tests `@smoke` in cookbook/web and cahokia/web, let `playwright.config.ts` use `BASE_URL` and record video, screenshots and traces for smoke runs, and add a `smoke` task to the web task files.
- TODO (agent-generated): Run the smoke tests against each PR preview and against production after each deploy, publish the Playwright HTML report to a `uat-reports` Cloudflare Pages project, and link it from the sticky PR comment.
- TODO (agent-generated): Add a UAT section (what changed for the user, what to click on the preview) to the PR template and the `open-pull-request` skill.
- TODO (agent-ignore): Create the PostHog Cloud project and add a read-only PostHog personal API key as a repo secret. Needs Jim.
- TODO (agent-generated): Add PostHog session replay to each production web app (production host only, inputs masked, console and network capture on) with a privacy note on each site.
- TODO (agent-generated): Add a `review-user-sessions` agent skill that reads the past week's PostHog sessions with errors, rage clicks or dead clicks and records `FIX`/`TODO` items with recording links in the right `ISSUES.md`, and ask Jim to schedule it weekly.
- TODO (agent-ignore): Create a Honeycomb account on the free tier and add `HONEYCOMB_API_KEY` as a repo secret. Needs Jim.
- TODO (agent-generated): Send GitHub Actions CI/CD runs to Honeycomb as traces, one span per job and step.
- TODO (agent-generated): Post a Honeycomb deploy marker (project, commit, PR) from every production deploy.
- TODO (agent-generated): Add Honeycomb's OpenTelemetry web SDK to each web app for page loads, Core Web Vitals and uncaught errors, using an ingest-only key.
- TODO (agent-generated): Export `tracing` spans over OTLP to Honeycomb from any Rust service with a backend, using `tracing-opentelemetry`.
- TODO (agent-generated): Add a Honeycomb trigger that alerts when browser errors rise after a deploy marker.

# Priority Projects
- cast
- cookbook
- homer
- controlled_chaos
- royalty
- sparkhill

# On Hold Projects
- luggage
- starcraft
- profiles