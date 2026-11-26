# 0003. Evaluate product requirements at every stage, on free tiers

- Status: Proposed (accepted when Jim approves the PR that adds it)
- Date: 2026-11-26
- Plan: [../plans/2026-11-eval-engine.md](../plans/2026-11-eval-engine.md)

## Context

Jim wants a robust eval engine built into the repo's processes: evaluate the
product requirements against what is actually implemented, after CI, after
CD, against observability data, with human feedback in pane, and with
automated chaos testing in production.

Today nothing ties a check to a product requirement, and no project writes
its requirements down. Tests pass or fail per project; the deploy-feedback
plan (decision 0001) adds smoke tests, observability and replay, and pane
(PR #606) adds a dashboard, but none of them say whether a given promise to
the user holds in production. Most changes are made by agents, so the
signal has to be automatic and has to reach the existing `ISSUES.md` loop.

Constraints: approved vendors only (GitHub, Cloudflare; PostHog for replay),
free tiers only with guards (`free-tiers.md`), all code in Rust, the repo as
the source of truth.

## Decision

1. **Requirements are written down per project, with stable IDs.** Each
   shipping project has `docs/requirements.md` (`## COOK-001: ...` with
   Priority, Why and Accept fields) and `evals.toml` listing the evals for
   each requirement. Requirements are Jim's product intent: a PR that adds
   or changes one is labelled `needs-human`.
2. **One engine, six stages.** A Rust project, `evals`, validates the files,
   collects results from existing test reports (Cargo, Playwright, trycmd)
   and runs HTTP, metric and fault evals itself, at stages `ci`, `cd`,
   `chaos`, `observe`, `judge` and `human`. It computes one verdict per
   requirement (met, at risk, not met, unknown) with the same code pane
   uses.
3. **Results are files in the monorepo, not a database** (Jim,
   2026-11-26: avoid databases for as long as possible; dev work stays as
   files in the repo). Each run writes its job summary and a JSONL artifact,
   and commits its records as `results/<project>/<yyyy-mm>.jsonl` plus a
   recomputed `verdicts.json` to a long-lived `evals-results` data branch
   that is never merged into `main`. Human feedback lives on `main`
   (`<project>/evals/feedback.jsonl`, written by pane's `pane-edit` PRs).
   If a query engine is ever needed, a SQLite file committed next to the
   JSONL comes first; a hosted database only after the repo-wide database
   evaluation.
4. **Production chaos is narrow and capped.** Faults are injected only in
   our own browser sessions (Playwright against production) and, with an
   HMAC-signed header, in our own Workers for that one request. No load
   testing; at most 500 Worker requests a night; third parties (Auth0,
   PostHog, GitHub, RapidAPI) are only ever blocked from the browser, never
   called; runs only after that night's `cd` evals pass; a repo variable
   `EVALS_CHAOS` is the kill switch and starts `off`.
5. **Failures feed the existing loop.** A daily `eval-triage` skill turns a
   requirement that is newly not met into a `FIX` in the project's
   `ISSUES.md`, and a broken or disputed eval into a `TODO`, in one draft
   PR. After CD, a failure fails the deploy job and offers a rollback, but
   nothing rolls back automatically yet.
6. **Agent judges are advisory, Jim's rating wins.** `judge` evals can make
   a requirement amber, never red. Their agreement with Jim's ratings is
   measured, and a rubric that disagrees too often is rewritten.
7. **Synthetic traffic is tagged and excluded** from Web Analytics,
   Analytics Engine metrics and PostHog recordings, so evals never grade
   themselves or spend the recording allowance.

## Alternatives considered

- **Cloudflare D1.** Free and on an approved vendor, and the first draft of
  this plan used it. Dropped at Jim's request: no database while files will
  do.
- **Results in Analytics Engine.** Already planned for `telemetry`, but it
  keeps data for 3 months and is built for metrics, not per-requirement
  records with evidence links. Kept for the metrics evals read.
- **Results committed to `main`.** A few hundred lines a night would need a
  PR or a direct push to `main` every run and bury real changes in noise.
  The data branch keeps the history in the same repo without that.
- **A hosted eval or chaos service** (Gremlin, Datadog Synthetics,
  Checkly). All paid past small free tiers or add a vendor; the browser and
  our Workers are enough for the faults that matter to our products.
- **Automatic rollback on a failed `cd` eval.** Safer in theory, but with
  new evals a flaky one would roll back good deploys. Revisit after a month
  of green `cd` runs.

## Consequences

- Every shipping project needs requirements written and approved before the
  engine says anything about it. Drafting them is agent work; approving them
  is a recurring ask of Jim, a few minutes per project.
- Low-traffic apps get few observability results (`min_samples` turns those
  into `skip`), so `cd` and chaos evals carry most of the weight until there
  are real users.
- Eval workflows need `contents: write` to push to `evals-results`. The
  repo check should make sure they push nowhere else.
- The Cloudflare API token needs Analytics Engine read, and a PostHog read
  key is needed, when Phase 3 starts: `TODO (human)` lines then.
- Pane's Health page gains a Requirements signal, and pane's write path
  gains feedback lines, so pane's later phases and this plan move together.
