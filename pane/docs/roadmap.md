# Pane roadmap

Where pane goes next, toward the vision in [mission.md](mission.md): one
simple dashboard of every action waiting on Jim and one scorecard per
domain, with drill-down into each project (Jim, 2026-11-28: "Now it has so
much information it is becoming hard to consume. We should add a product
roadmap in which the long term vision is to have a simple high level
dashboard of all relevant actions and domain scorecards with the ability to
drill down into projects."). The format follows the product strategy plan
(PR 922). Each line links the issue file that does the work; the review
dashboard plan, [plans/2026-11-review-dashboard.md](plans/2026-11-review-dashboard.md),
holds the architecture.

Pane today has eleven pages, each good at one thing. The roadmap doesn't
add pages: it puts a summary on top of them and makes the rest drill-downs.

## Product roadmap

### Now

- **Repo health summary.** Today opens with the README's Repo health
  badges, grouped into domains, and the Health page drills into each check:
  what it runs, what red means and the pane page with the detail. The README
  tables and pane both read `standards/repo-health.toml`, so they can't
  disagree ([PANE-014](requirements.md)).
- **One list of actions on Today.** Human issues, PRs to approve, merged
  PRs to acknowledge and decks to sign off in one list, oldest first, each
  with one action ([today-one-needs-you-list](../issues/today-one-needs-you-list.md)).

### Next

- **Domain scorecards.** One card per domain with a light, a headline
  number and the projects that make it worse, opening a domain page and
  then the Project page
  ([compute-domain-scorecards](../issues/compute-domain-scorecards.md),
  [show-domain-scorecards-today](../issues/show-domain-scorecards-today.md)).
- **Live pull requests.** PRs and their checks on Today and Project pages
  from GitHub, not only from the last snapshot
  ([show-live-data-app-prs-today](../issues/show-live-data-app-prs-today.md)).
- **Fewer pages in the nav.** Today, Projects and Health up top, the rest
  behind More and reachable from the dashboard
  ([simplify-pane-nav](../issues/simplify-pane-nav.md)).

### Later

- **Traffic lights per project** on the Health page, behind each scorecard
  ([build-health-page-grid-projects-systems](../issues/build-health-page-grid-projects-systems.md)).
- **Two weeks of history** for each light, so a scorecard shows its trend
  ([keep-last-14-daily-health-results](../issues/keep-last-14-daily-health-results.md)).
- **Act without leaving pane**: add a TODO, close a human issue, rate a
  requirement ([build-page-human-issue-todo-fix](../issues/build-page-human-issue-todo-fix.md),
  [rate-met-not-met-not-sure](../issues/rate-met-not-met-not-sure.md)).

## Technical roadmap

### Now

- **One source per number.** A dashboard figure that also appears
  elsewhere (the README, the audit, the eval badge) is read from the same
  file, never recomputed differently. The repo health badges are first:
  `standards/repo-health.toml`, written into the README by `chores
  repo-health` and checked by the audit (HLT-001)
  ([PANE-014](requirements.md)).

### Next

- **Scorecards in `pane_model`.** Scorecard rules are pure functions over
  the snapshot with thresholds in `pane/health.toml`, like the existing
  signals, so they are tested without a browser
  ([compute-domain-scorecards](../issues/compute-domain-scorecards.md),
  [compute-system-signals-nightly-ci-cd](../issues/compute-system-signals-nightly-ci-cd.md)).
- **Sign-in and live GitHub data.** Cloudflare Access in front of pane,
  then cached `/api/github/*` routes for workflow runs and PRs, so badge
  colours can feed the scorecards instead of only being shown
  ([put-pane-behind-cloudflare-access-allowing](../issues/put-pane-behind-cloudflare-access-allowing.md),
  [api-github-routes-pane-worker-cached](../issues/api-github-routes-pane-worker-cached.md)).

### Later

- **History in Workers KV**, one write a day, for the trends
  ([keep-last-14-daily-health-results](../issues/keep-last-14-daily-health-results.md)).
- **Telemetry routes** for the observability scorecard
  ([api-telemetry-routes-query-analytics-engine](../issues/api-telemetry-routes-query-analytics-engine.md)).
