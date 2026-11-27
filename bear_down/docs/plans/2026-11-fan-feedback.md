# Plan: fan feedback and an A/B test of the "doesn't matter" line

- Status: accepted (Jim, 2026-11-26): Cloudflare `telemetry` Worker; arm B
  at 0.02 points and the optional note are the defaults Jim didn't change
- Requirements: BEAR-005 (the 0.05-point line), new BEAR-010 below
- Builds on: [the deploy-feedback plan](../../../standards/docs/plans/2026-11-deploy-feedback.md)
  (UAT, the `telemetry` Worker, PostHog) and
  [the eval engine plan](../../../standards/docs/plans/2026-11-eval-engine.md)
  (evals, synthetic traffic tagging)

## Why

Jim, 2026-11-26: "The 0.05 line is fine for now. Need user feedback on that.
Can we add some user feedback options? Maybe leveraging A/B via our UAT
design?"

The line decides which games say "doesn't matter" (BEAR-005). We picked 0.05
Super Bowl points by judgment. Fans are the ones who can tell us if a
"doesn't matter" game really felt like it didn't, or if the explanations
make sense, so we ask them, and we try a second line on half of them.

## What fans see

Under each game's explanation, three small buttons and nothing else:

> Does this verdict look right? **Yes** · **No** · **Confusing**

After a tap, the buttons are replaced by "Thanks" and an optional one-line
box ("What's off?", 200 characters, not required). One vote per game, per
tab, per week, per browser. No sign-in, no cookies.

## The A/B test

- **What varies:** the "doesn't matter" line. Arm A keeps 0.05 Super Bowl
  points. Arm B uses 0.02 points, so more small games become picks with
  their channels shown. (Alternatives Jim may prefer: 0.1 points, or a line
  relative to the Bears' odds, such as 1% of them.)
- **How:** no flag service. `rooting.json` gains each swing's `std_error`,
  and the page computes the tier for its arm in the browser with the same
  rule as the CLI (a shared function in `bear_down_core`, tested against
  the CLI's tiers). The arm comes from a random id the page keeps in
  `localStorage`: the same browser stays in the same arm all season.
  `?arm=a` or `?arm=b` forces one, for UAT and for Jim.
- **What we measure, per arm:** the share of "No" votes on "doesn't matter"
  verdicts and on picks, the share of "Confusing" votes, and votes per
  visit. Arm B wins if it lowers "No" on "doesn't matter" verdicts without
  raising "Confusing".
- **When we decide:** after 200 votes per arm or at the end of the regular
  season, whichever comes first. With a site this small that may not
  happen; then the notes and Jim's judgment decide, and the plan says so in
  its retro rather than overreading a handful of votes.

## Where the data goes

Cloudflare only, which is already approved and free:

- The page posts `exposure` (arm, week, tab) and `feedback` (arm, week, tab,
  game id, verdict shown, vote, optional note) events to the `telemetry`
  Worker from the deploy-feedback plan, phase 5. Building that Worker is
  already a root backlog TODO; this plan does it, with these two event
  kinds added to `error`, `game`, `cli` and `deploy`.
- The Worker validates and rate-limits (one event per second per IP, notes
  cut to 200 characters and stripped of anything that looks like an email
  or phone number) and writes to Analytics Engine: 100,000 data points a
  day free, far beyond our traffic.
- Files before databases: a weekly GitHub workflow queries Analytics Engine
  and commits `bear_down/<season>-week-<nn>.jsonl` (one line per event) and
  a short Markdown summary per arm to the `feedback-data` data branch. The repo stays the record;
  Analytics Engine is only the inbox.
- Synthetic traffic (UAT and eval runs) sends `x-monorepo-synthetic: 1`, and
  the Worker drops it, per eval engine phase 3.

Why not PostHog: its free plan has surveys (1,500 responses a month) and
feature flags (1M requests), which would also work. But PostHog is approved
only for session replay, and it isn't in any project yet. Using it here
would widen that exception and add a script to the page. If Jim prefers
PostHog, steps 2 and 4 swap to its survey and flag APIs and the rest stays.

## UAT and evals

- Playwright `@smoke` tests load the page with `?arm=a` and `?arm=b` and
  screenshot both, so each PR's UAT report shows what each arm sees. A test
  votes on a game and checks the "Thanks" state, against a stubbed
  `telemetry` endpoint.
- New requirement, for Jim to approve:

  > **BEAR-010: Fans can say when a verdict looks wrong.** Priority: should.
  > Why: the "doesn't matter" line and the explanations were set by
  > judgment, and fans are the ones who can tell (Jim, 2026-11-26).
  > Accept: every game in every tab has a Yes / No / Confusing vote with an
  > optional note, votes reach the repo weekly as files, and the summary
  > shows each A/B arm's results.

- `evals.toml`: a `ci` Playwright eval for the buttons, a `cd` `http` eval
  that the Worker accepts a synthetic event, and an `observe` `metric` eval
  that at least one real vote arrived in the past 14 days during the season.

## Privacy

No cookies and no sign-in. The random id lives only in the browser and is
never tied to anything else. Notes are optional and filtered. A one-line
note under the buttons says what is sent. Do Not Track turns sending off.

## Cost

Free: Pages (static), one Worker (100,000 requests a day, rate-limited),
Analytics Engine (100,000 writes a day), GitHub Actions for the weekly
export. No new vendor.

## Order of work

Each is one PR:

1. `std_error` in `rooting.json`, and the tier rule shared by the CLI and
   the page (no visible change). Done: `bear_down_core::tier_against`,
   used by the CLI and by the page's `Goal::verdict`.
2. The `telemetry` Worker with `exposure` and `feedback` events (the root
   backlog's Worker TODO), deployed with `free-tiers.sh` guards. Built in
   the root `telemetry/` project, with the other four kinds too.
3. The vote buttons and note box on the page, with the privacy note and
   `@smoke` tests. Done: a vote posts at once; the optional note follows
   as a second `feedback` event with `vote` set to `note`, so each vote
   counts once. The arm is `a` until step 4.
4. The A/B arms (`localStorage` id, `?arm=`), with screenshots of both arms
   in UAT. Done: a page in arm B that turns a game into a pick
   explains it with a "small edge" line over the 0.02-point line, and each
   visit sends one `exposure` per arm, week and tab (`sessionStorage`).
5. The weekly export workflow and per-arm summary, and the evals. Export
   done: `telemetry-feedback-export.yml` runs Tuesdays and writes
   `bear_down/<season>-week-<nn>.jsonl` and a per-arm `.md` summary with
   `standards/scripts/feedback.sh`. They go to the `feedback-data` data
   branch, not to `main`, because CI may never write `main` (SEC-007); the
   branch is in the repo, like `evals-results`. BEAR-010 and its evals
   follow in their own requirement PR, which Jim approves.
6. After the decision point, a retro in this file and, if arm B wins, a
   BEAR-005 change for Jim.

## Decisions

1. Cloudflare `telemetry` Worker, not PostHog (Jim, 2026-11-26). Both cost
   $0; Cloudflare has about 100 times the free-tier headroom and needs no
   change to the vendor approvals.
2. Arm B's line: 0.02 points (default; Jim may change it before step 4).
3. The optional one-line note is allowed (default).
