# Evals product roadmap

Written 2026-11-28 from the
[evals review deck](../../docs/reviews/2026-11-27-evals/deck.md), at Jim's
request:

> I just reviewed the eval review slide deck and I agree with everything.
> Lets take it's findings and create a product roadmap and associated todos
> in the eval project.

It also follows his answers to the deck's questions (2026-11-27): evals are
continuous improvement, not a one-off project; they should always be green;
every requirement that isn't green becomes a `FIX` or `TODO` issue; each
project grows its own coverage, shown as an eval coverage figure and badge;
triage is a free workflow; and the in-app feedback toolbar stays.

The design is still the
[eval engine plan](../../standards/docs/plans/2026-11-eval-engine.md) and
[decision 0003](../../standards/docs/decisions/0003-eval-engine.md). This
roadmap replaces the plan's phase order for the work that is left: the
phases there are history, and the work is now ordered by what makes the
lights trustworthy soonest.

## Is there an eval project?

Yes and no. There is one project, [`evals/`](../README.md), that holds the
engine, but by design most of the eval process lives next to what it
checks:

| Piece | Where it lives |
| --- | --- |
| The engine: types, parsers, verdict rules (`evals_model`) and the `evals` command (`evals_cli`) | [`evals/model`](../model), [`evals/cli`](../cli) |
| Verdict thresholds | [`evals/verdicts.toml`](../verdicts.toml) |
| Each project's promises and checks | `<project>/docs/requirements.md` and `<project>/evals.toml` (12 projects on 2026-11-28) |
| The format and rules | [`standards/docs/evals.md`](../../standards/docs/evals.md) |
| CI and deploy wiring | `.github/workflows/mise-projects.yml`, `push-main.yml`, `nightly-ci.yml`, `nightly-cd.yml`, and `standards/scripts/evals.sh` and `evals-check.sh` |
| Storing results, triage and the fire drill | `.github/workflows/evals-store.yml`, `evals-triage.yml`, `evals-fire-drill.yml` |
| Results and verdicts | the `evals-results` branch (`results/*.jsonl`, `verdicts.json`, `badges/`) |
| Issues the verdicts raise | `<project>/issues/eval-<id>-*.md`, written by triage |
| Hand-made settings checks | `standards/settings.toml`, `standards/settings-checks.jsonl` |
| Showing the lights | pane's Requirements view (planned, `pane/issues/`) |
| Other readers | `cost/cli/src/evals.rs` |

That split is deliberate: a project owns its promises, and the engine only
reads them. What was scattered was the to-do list. The engine's own issues
were in the root `issues/`; they are now in [`evals/issues/`](../issues/),
under the epic "Evals roadmap". Work that belongs to one project (its
requirements, its smoke tests, its production evals) stays in that
project's `issues/` with the same epic, so it is listed with the roadmap.

## Where it stands, 2026-11-28

From `verdicts.json` on the `evals-results` branch (generated 03:01 UTC):

| | Review (2026-11-27) | Today |
| --- | --- | --- |
| Projects with requirements | 4 | 12 of 27 |
| Requirements | 35 | 92 |
| Green (met) | 12 | 29 |
| Amber (at risk) | 1 | 11 |
| Red (not met) | 0 | 1 (controlled_chaos) |
| Grey (unknown) | 22 | 51 |
| Repo eval coverage | none | 13.4% (musts 37.5%) |

What the deck's recommendations led to:

| # | Recommendation | State |
| --- | --- | --- |
| 1 | Playwright smoke tests against production; content checks | Partly done: cookbook (PR 861) and Blue Eel run smoke tests after deploy, and `body_contains`, `asset_contains` and the EVL-006 warning landed (PR 835). bear_down, pane and cahokia don't run theirs yet, and EVL-006 is still a warning |
| 2 | Requirements for Blue Eel, then controlled_chaos and the CLIs | Done for Blue Eel (sparkhill), controlled_chaos, homer, my_lang, design_system, telemetry, royalty and tofu_state. 10 projects still have none |
| 3 | Triage on a free workflow | Done: `evals triage` and the daily `evals-triage.yml` (PRs 841, 842). It hasn't filed anything yet |
| 4 | One fire drill | Done: [retro](fire-drill-2026-11-27.md) and the repeatable `evals-fire-drill.yml`. Two of its findings are still open (below) |
| 5 | Read-only Requirements page in pane | Not started (`pane/issues/requirements-view-page-requirement-verdict-evals.md`) |
| 6 | "Every must has a production check" as an error | Not yet: 16 musts have none (EVL-004) |
| 7 | Pause the in-app toolbar | Jim said no. It moves to the feedback component instead (below) |

So the lights now cover most products, but most of them are grey, and the
green still leans on checks that are shallower than they look.

## Goal and measures

**Goal:** every requirement of every active project is green, on evidence
from production, and anything that isn't green is already an issue someone
is working.

| Measure | Today | Target |
| --- | --- | --- |
| Active projects with requirements | 12 of 22 (on-hold and tooling left out) | all |
| Musts met | 37.5% | 100% |
| Requirements green | 29 of 92 (32%) | over 80% (the coverage warning line) |
| Non-green requirements with an open issue | none filed yet | 100%, filed by triage within a day |
| Production evals that check content | 9 status-only warnings (EVL-006) | 0, and EVL-006 an error |

## Product roadmap

The headings follow decision 0022's roadmap format (Now, Next, Later), with
the four phases inside them. Four phases, worked in order, with each project's own issues worked
alongside. "Effort" is Claude Code build effort (S under an hour, M a few
hours, L a day or more), which counts against the $25 a month budget. Every
item costs $0 in money.

### Now

#### Phase 1: Trust the green

A green light must mean "works in production", and a red one must become
work. These are `priority` issues.

| Order | Issue | Effort |
| --- | --- | --- |
| 10 | [Count musts with production results, not evals](../issues/count-musts-with-production-results.md) | S |
| 20 | [A build failure leaves no eval result](../issues/build-failure-leaves-no-result.md) | S |
| 30 | [EVL-006 should accept header checks](../issues/evl-006-counts-header-checks.md) | S |
| 40 | [Make EVL-006 an error](../issues/evl-006-an-error.md) | S |
| 50 | [Review the first triage runs](../issues/review-the-first-triage-runs.md) | S |
| 60 | [bear_down smoke tests after deploy](../../bear_down/issues/smoke-tests-after-deploy.md) | S |
| 70 | [pane smoke tests after deploy](../../pane/issues/smoke-tests-after-deploy.md) | S |
| | [cahokia/web smoke tests](../../issues/smoke-uat-tag-few-playwright-tests.md) (already filed, "Feedback on every change") | S |
| | [pane's Requirements view](../../pane/issues/requirements-view-page-requirement-verdict-evals.md) (already filed) | M |

### Next

#### Phase 2: Cover every project

Every active project states its promises and checks each must in
production. Requirement PRs need Jim's approval, so these land at his pace.

| Order | Issue | Effort |
| --- | --- | --- |
| 5 | [Leave on-hold projects out of the coverage figure](../issues/leave-on-hold-out-of-coverage.md) | S |
| 10 | Requirements for [evals](../issues/write-requirements.md), [blueeel](../../blueeel/issues/write-requirements.md), [auth_sdk](../../auth_sdk/issues/write-requirements.md), [cost](../../cost/issues/write-requirements.md), [threat](../../threat/issues/write-requirements.md), [chores](../../chores/issues/write-requirements.md), [infra](../../infra/issues/write-requirements.md), [psy_ents](../../psy_ents/issues/write-requirements.md), [athenaeum](../../athenaeum/issues/write-requirements.md) and [bogan](../../bogan/issues/write-requirements.md) | S each |
| 60 | [Make EVL-004 an error](../issues/evl-004-an-error.md) | S |
| 70 | [`trycmd` against release binaries, and the `script` kind](../issues/trycmd-cd-and-script-kind.md) | M |
| 80 | [design_system DSGN-002 production eval](../../design_system/issues/dsgn-002-production-eval.md) | S |
| 90 | [my_lang production evals with the CLI](../../my_lang/issues/production-evals-with-the-cli.md) | S |
| 100 | [tofu_state `cd` evals after deploy](../../tofu_state/issues/run-cd-evals-after-deploy.md) | S |

royalty's nine musts get production evals as its Phase 1 code ships (its
`docs/requirements.md` says so), and PANE-013 gets one through triage.

### Later

#### Phase 3: Deeper evidence

Beyond "the page has the right content": real-user numbers, and people's
judgment.

| Order | Issue | Effort |
| --- | --- | --- |
| 10 | [The `metric` eval kind and the daily observe run](../issues/metric-eval-kind.md) | L |
| 15 | [telemetry TELE-004 production eval](../../telemetry/issues/tele-004-production-eval.md) (needs the `metric` kind) | S |
| 20 | [Read Jim's ratings in the verdicts](../issues/read-jims-ratings-in-verdicts.md), with pane's [Rate action](../../pane/issues/rate-met-not-met-not-sure.md) | M |
| 40 | [Read feedback comments in the verdicts and triage](../issues/read-feedback-comments-in-verdicts.md) (needs the feedback component, below) | M |

#### Phase 4: Stress and judgment

| Order | Issue | Effort |
| --- | --- | --- |
| 10 | [Requirements touched by a PR, on its preview](../issues/preview-requirements-in-pr-comment.md) (needs preview deploys) | M |
| 20 | [The `fault` kind and nightly chaos](../issues/fault-eval-kind-and-nightly-chaos.md) | L |
| 30 | [FT-006 chaos guard](../issues/ft-006-chaos-guard.md) | S |
| 40 | [Signed chaos header](../issues/signed-chaos-header.md) | M |
| 50 | [CI-only fault cases](../issues/ci-only-fault-cases.md) | M |
| 60 | [Agent judge for taste-type promises](../issues/agent-judge.md) | M, then weekly Claude use |
| 70 | [Process evals (`JIMH-xxx`)](../issues/process-evals.md) | M |

## Technical roadmap

The engine's own health, apart from what it checks.

### Now

- [Replace or justify chrono, flagged unmaintained](../../issues/threat-check-advisories-evals.md) (S).
- [Write the evals user guide](../issues/write-user-guide.md) (S).

### Next

- Port `standards/scripts/evals.sh` and `evals-check.sh` into `evals`
  subcommands, under the code quality rule that scripts stay Rust-only
  ([decision 0015](../../standards/docs/decisions/0015-code-quality-process.md)),
  as the [code quality epic](../../issues/code-quality-phase-2-port-shell-scripts.md)
  reaches them (M).

### Later

- Share `evals_model` with pane's Requirements view so both apply the same
  verdict rules (part of the pane issue above).

## The feedback toolbar: one tool, chosen by who is signed in

Jim's comment on the deck (2026-11-28):

> The one major comment I have is that the vercel like plugin may fit in
> better with the feedback project. Maybe it's a common tool but depending
> on who is logged in, you can give user feedback, app manager feedback,
> product owner feedback or engineering feedback?

**Recommendation: yes.** The eval engine should consume feedback, not
build the tool that captures it. The
[user feedback plan](../../standards/docs/plans/2026-11-user-feedback.md)
(PR 915, decisions 0020 and 0021) already moves the review
toolbar's model into the `feedback` project; this goes one step further and
makes it one tool with one entry point, where the role decides what the
form offers and where it goes:

| Role | Who | What they can send | Where it goes |
| --- | --- | --- | --- |
| User | anyone, not signed in | the privacy-safe form: topic, rating, note, exactly what they typed (no screenshot, no page contents) | the product's own inbox, in its tenant tables |
| App manager | signed in with the management app role (decision 0018) | the user form plus the flag or setting it is about | the product's own inbox, marked for the app manager |
| Product owner | signed in as the product's reviewer (decision 0019) | pins on the page tied to a requirement ID, and met, not met or looks good ratings | `<project>/evals/feedback.jsonl`, which the verdicts read (Phase 3) |
| Engineering | signed in as an engineer | pins with selector, screenshot, console errors and the build | a `FIX` or `TODO` issue file through triage |

What this changes and what it keeps:

- **One component, one button.** `feedback/dioxus` (and the Bevy plugin
  for games) shows one Feedback button. Not signed in, it is the user
  form. The richer modes appear only after a sign-in that returns a role.
- **The user's privacy rule holds.** The staff modes' code (screenshots,
  DOM reading, console errors) loads only after a staff sign-in, so a
  user's page, Blue Eel's included (SPRK-006: no tracking, no third-party
  script), never runs it.
- **Products stay separate.** User and app manager feedback stays in the
  product's own data; product owner and engineering feedback goes to the
  repo, which is the product team's own tooling.
- **The eval engine's part shrinks to reading.** The phase 6 capture
  issues (model, build meta tag, web toolbar, Bevy plugin, cahokia
  webviews, rollout) move to the "User feedback" epic, Phase 8 (PR 928).
  Only the verdict side, Phase 3 order 40 above, stays here.
- **Cost:** $0 in money. Auth0 roles are on its free plan and the sign-in
  is the one auth_sdk already does. Claude effort is about the same as the
  separate toolbar, since the forms share one model and one widget.

PR 915 (decisions 0020 and 0021) merged on 2026-11-28; PR 928 adds this
as item 7 of decision 0020, "One Feedback button, four roles", and moves
the phase 6 issues.

## How the roadmap stays current

- Triage files the per-requirement work every day; this roadmap holds only
  the engine and coverage work.
- Close an issue by deleting its file in the PR that does it, and tick it
  off here in the same PR.
- The next evals review (the deep project review rotation) checks the
  measures above against the targets.
