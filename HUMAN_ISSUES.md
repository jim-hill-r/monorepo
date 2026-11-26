# Human Issues

Tasks that only a person can do: anything in a vendor dashboard, an account,
a secret or a payment setting, and sign-offs an agent may not give itself.
Agents never work these. A weekly scheduled task (the `human-issues-digest`
skill) brings the open ones to Jim's attention in the Claude project. The
process is in
[standards/docs/issue-management.md](standards/docs/issue-management.md#human-issues).

Each line says what to do, where, and why, and ends with when it was added
and what it unblocks. Remove a line (in a PR) once it is done; any agent will
do that when told it is done.

# Open

- TODO (human): In the Auth0 dashboard, add the Cloudflare Pages preview domains to the allowed callback URLs for cookbook and cahokia. (Added 2026-11-25. Unblocks sign-in on PR previews, from the preview deploys TODO in the root ISSUES.md.)
- TODO (human): Create the PostHog Cloud project (free plan, no card) and add a read-only PostHog personal API key as a repo secret. (Added 2026-11-25. Unblocks PostHog session replay and the `review-user-sessions` skill, decision 0001.)
- TODO (human): In the Cloudflare dashboard, turn on Web Analytics for each production Pages project and give the `CLOUDFLARE_API_TOKEN` token Analytics Engine read access. (Added 2026-11-25. Unblocks the telemetry TODOs in the root ISSUES.md.)
- TODO (human): Check there is still no payment method on the Cloudflare, PostHog, Auth0 and RapidAPI accounts, and that GitHub's Actions budget is $0 with usage stopped at the limit. (Added 2026-11-26, PR #582. Keeps everything on free tiers, see standards/docs/free-tiers.md.)
- TODO (human): In the Cloudflare dashboard, give the `CLOUDFLARE_API_TOKEN` secret's token "Workers Scripts: Read" (it already has Pages edit). (Added 2026-11-26, PR #582. Lets the weekly All Free Tier Usage run count Workers instead of skipping them.)
- TODO (human): In the Auth0 dashboard, open the application Cahokia uses (client ID `6CHDECRfCsyYdCFq1hwqKNwCHxxmum3E`): add `https://cahokia-web.pages.dev/` and `http://localhost:8080/` to Allowed Callback URLs, add `https://cahokia-web.pages.dev` and `http://localhost:8080` to Allowed Web Origins and Allowed Logout URLs, and check the application type is Single Page Application. (Added 2026-11-26. Unblocks sign-in on Cahokia production, see cahokia/docs/plans/2026-11-prod-oidc-error.md.)
- TODO (human): In GitHub, Settings > Rules > Rulesets > "Default" (the ruleset on `main`), turn on "Require status checks to pass" and add the `CI result` check (GitHub Actions). (Added 2026-11-26. Makes auto-merge wait for green CI after any push; until then agents only turn auto-merge on while checks are green, see the Auto-merge section of standards/docs/pull-requests.md.)
