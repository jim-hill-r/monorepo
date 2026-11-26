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
3. **Results go to Cloudflare D1**, database `evals`, plus each run's job
   summary and a JSONL artifact. D1 is new to the repo but is on an approved
   vendor and free at our scale (under 5,000 writes a day of 100,000).
   Human feedback is the exception: it lives in the repo
   (`<project>/evals/feedback.jsonl`, written by pane's `pane-edit` PRs) and
   is copied into D1.
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

- **Results in Analytics Engine.** Already planned for `telemetry`, but it
  keeps data for 3 months and is built for metrics, not per-requirement
  records with evidence links. Kept for the metrics evals read.
- **Results committed to the repo.** Fits "repo as source of truth", but a
  few hundred rows a night would bury real changes in noise. Only Jim's own
  feedback, which is low volume and is a decision, is committed.
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
- The Cloudflare API token needs D1 Edit, and later Analytics Engine read and
  a PostHog read key: `TODO (human)` lines when those phases start.
- Pane's Health page gains a Requirements signal, and pane's write path
  gains feedback lines, so pane's later phases and this plan move together.
