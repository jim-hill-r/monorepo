# telemetry: isolation

How telemetry would work outside this repository, and what it shares with
other projects ([project isolation](../../standards/docs/project-isolation.md)).

## Moving out

telemetry is a self-contained Rust Worker. A copy of this folder needs:

1. **Tasks.** `mise.toml` includes `../standards/tasks/cloudflare-worker.toml`.
   Copy that file into the new repo (or inline its tasks), and copy
   `standards/scripts/free-tiers.sh`, which `mise run deploy` runs first, or
   drop that step if the new owner has no shared free-tier budget to guard.
2. **Workflows.** CI and deploys run from the shared
   `.github/workflows/pull-request-ci.yml` and `push-main.yml`; a new repo
   needs one workflow that runs `mise run ci` on pull requests and
   `mise run deploy` on `main`. The weekly vote export,
   `telemetry-feedback-export.yml` with `standards/scripts/feedback.sh`,
   belongs to bear_down's fan feedback and moves with bear_down instead.
3. **Secrets.** `CLOUDFLARE_API_TOKEN` and `CLOUDFLARE_ACCOUNT_ID` (see the
   Cloudflare account below).
4. **Badges and evals.** The README's eval coverage badge points at this
   repo's `evals-results` branch; remove it or point it at the new repo's
   own results. `evals.toml` needs the `evals` CLI.

Nothing else in this repo depends on telemetry's code.

## Shared resources

telemetry owns `telemetry-worker`, so it does not list it here. Its users
are bear_down (`web/src/feedback.rs`) and controlled_chaos
(`src/telemetry.rs`); each documents how to separate from it in its own
`docs/isolation.md`. Events are keyed by `project`, so one user's events
can be exported or dropped without touching the others.

### Cloudflare account (`cloudflare-account`)

#### Why it is shared

Every project's Workers and Pages run in one free Cloudflare account, so
there is one deploy token to mint and rotate, one set of free-tier budgets
to guard, and one `jim-hill-r.workers.dev` subdomain. A second account
would cost nothing but double the credentials and checks. telemetry uses
Workers (one script), Analytics Engine (the `telemetry` dataset) and a
rate-limit binding, all on the free plan.

#### Migration guide

1. Create a free Cloudflare account for the new owner (no payment method).
2. Create an API token with Workers Scripts:Edit and Account Analytics:Read,
   for that account only.
3. Set `CLOUDFLARE_API_TOKEN` and `CLOUDFLARE_ACCOUNT_ID` as secrets in the
   new repo.
4. Run `mise run deploy`. The Worker is created as `jimhillr-telemetry` on
   the new account's own `*.workers.dev` subdomain (rename it in
   `wrangler.toml` if the new owner prefers), and the dataset is created on
   the first write.
5. Update `DEPLOY_URL` in `mise.toml`, and tell each user project to point
   its client at the new URL.
6. Export any events still needed from the old dataset (the weekly export
   already copies bear_down's to the `feedback-data` branch; Analytics
   Engine keeps data for 3 months), then retire the old Worker in
   `standards/resources.toml`.

Cost after separating: $0 (Workers Free: 100,000 requests a day;
Analytics Engine: 100,000 data points a day). Verify: `POST /event` with a
valid event answers 202, and `mise run evals` passes against the new URL.
