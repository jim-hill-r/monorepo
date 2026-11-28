# The eval process: executive review, 2026-11-27

Requested by Jim on 2026-11-27: "Can you do an executive review for the eval
process? ... I want to understand this better." The slides are in
[deck.md](deck.md), built from this review. Reviewer: Claude.

Every figure below comes from the repo or the `evals-results` branch as of
2026-11-27 13:26 UTC (the newest `verdicts.json`). Guesses are marked as
guesses.

## In one paragraph

The eval process answers one question for each product: **does what we
shipped still do what we said it should?** Each project writes its promises
down as numbered requirements, ties each one to automatic checks ("evals"),
and runs those checks after every build and every deploy. The results are
saved as plain files and turned into a traffic light per requirement. Two
days in, the bones work: 4 projects have 35 requirements and 88 evals, every
push to `main` records results, and 12 requirements show green. But the
green is thinner than it looks. The production checks mostly prove a page
loads, not that it works, and nothing has ever failed, so we have not yet
seen the loop catch a real problem. The biggest gaps are coverage (Blue Eel,
the top priority, has no requirements), depth (no browser test runs against
production yet) and the feedback loop (nothing turns a red light into work
yet).

## Why we have it

Tests tell us the code does what the code says. They don't tell us the
product does what *Jim* wants, in production, for real users. Before this
process, "what Jim wants" lived in READMEs, plans and review decks, and
nothing checked it after a deploy. The eval process makes that intent
explicit and checkable, and gives every project the same simple light:
green (met), amber (at risk), red (not met) or grey (unknown).

The design is in the
[eval engine plan](../../../evals/docs/plans/2026-11-eval-engine.md) and
[decision 0003](../../../standards/docs/decisions/0003-eval-engine.md), both
approved on 2026-11-26. The day-to-day rules are in
[standards/docs/evals.md](../../../standards/docs/evals.md).

## How it works, end to end

1. **Write the promise.** A project lists its requirements in
   `docs/requirements.md`, each with an ID (`COOK-001`), a priority (must,
   should or could), why it matters and what "met" means. Agents draft
   them; Jim approves them, because they define what "working" means.
2. **Tie checks to it.** `evals.toml` lists the evals for each requirement:
   an existing Rust or Playwright test, a single web request to the live
   site, or (later) a metric, an injected fault, an agent's judgment or
   Jim's own rating.
3. **Check the files.** `mise run //:audit` refuses an eval that points at a
   missing requirement or a test that doesn't exist (checks EVL-001 to
   EVL-004).
4. **Run after CI.** On every PR, push and nightly build, the `evals` step
   reads the test reports the build already wrote (tests run only once) and
   turns them into one result per eval.
5. **Run after deploy.** Right after a production deploy, the live site is
   checked. A failing check on a must requirement turns the deploy job red.
6. **Store the results.** Pushes to `main` and nightly runs append results to
   files on a separate `evals-results` branch and rewrite `verdicts.json`. No
   database, nothing on `main` changes.
7. **Compute the light.** The same Rust code computes each requirement's
   verdict for the CLI and, later, for pane. A requirement is green only if
   all its evals pass *and* a production check is under 2 days old.
8. **Turn red into work** (not live yet). A daily agent, `eval-triage`, will
   turn a newly red requirement into a `FIX` issue file, so the normal agent
   loop picks it up.

Settings made by hand in dashboards (Auth0 apps, tokens, connectors) are
covered too: 36 are listed in `standards/settings.toml`, and the weekly
human digest asks Jim to confirm the ones that are due, recording his "ok"
as a result.

## Where it stands

### By phase

| Phase | What it adds | State |
| --- | --- | --- |
| 0 Foundations | Formats, `evals check`, CI results | Done (PR #612 and follow-ups) |
| 1 After CI | Results branch, verdicts, pane and cahokia requirements | Done, except the PR-comment list, which waits for preview deploys |
| 2 After deploy | Live `http` checks, CLI checks, daily triage | Partly done: live checks merged (#755); CLI checks (#771) and triage (#772) are open PRs; "every must has a production check" is still a warning |
| 3 Observability | Real-user metrics (speed, errors) as evals | Not started; the `telemetry` Worker already drops tagged test traffic |
| 4 Chaos | Faults injected in our own sessions in production | Not started |
| 5 Ratings in pane | Jim rates requirements in pane | Not started |
| 6 In-app comments | A feedback toolbar in every app and game | Not started (the biggest phase, 8 issues) |
| 7 Agent judge | An agent grades taste-type requirements weekly | Not started; cookbook already has one judge eval waiting |
| 8 Process evals | Lead time, failure rate, review load | Not started |

All 20 remaining eval engine issues are in the backlog, none in priority.

### By project

| Project | Requirements (musts) | Evals | Green | Amber | Grey |
| --- | --- | --- | --- | --- | --- |
| bear_down | 9 (6) | 17 | 6 | 0 | 3 |
| cahokia | 7 (2) | 25 | 1 | 1 | 5 |
| cookbook | 9 (3) | 15 | 2 | 0 | 7 |
| pane | 10 (2) | 31 | 3 | 0 | 7 |
| **Total** | **35 (13)** | **88** | **12** | **1** | **22** |

- Every must requirement has at least one production check on paper, but
  cookbook's COOK-005 (accessibility) is still grey: its production check is
  a Playwright test, and no project runs Playwright against production yet.
- Cahokia's CAHO-003 (sign-in) is amber only because its Auth0 setting has
  never been checked by hand. The first weekly digest that asks will clear
  it.
- Most should and could requirements are grey because they have only CI
  evals, and the rules deliberately refuse green without a production
  result.
- No requirements yet for Blue Eel (`sparkhill/webapp`, the top priority),
  `sparkhill/website`, `blueeel`, `controlled_chaos`, `design_system`,
  `telemetry` or the CLIs (`homer`, `my_lang`). A bear_down addition
  (BEAR-010, PR #812) is waiting for Jim.

### Results so far

1,780 results stored since 2026-11-26 23:11 UTC, across 41 runs: 1,727 from
CI and 53 from production. **Every one of them passed.** That is good news
for the products, but it also means the process has not yet proven it
catches anything.

## What is working

- **One simple question, one light per promise.** The verdict rules are
  strict in the right direction: no production evidence means grey, never
  green, and a setting nobody checked is amber, never fine.
- **It reuses what exists.** Evals read the test reports CI already writes,
  so nothing runs twice and most evals are existing tests with an ID.
- **Plain files, no new vendor.** Results live in git on a branch, readable
  on GitHub, by pane and by agents. It costs $0 a month in money.
- **Guard rails are in the build.** The audit rejects evals that point at
  nothing, and a failed must check after a deploy turns `main` red.
- **Jim keeps the pen on intent.** Requirements need his approval; agents
  only draft them.

## Gaps and risks

1. **Shallow production checks (biggest risk).** All 53 production results
   are single web requests. Most check only that a page answers, such as
   `GET /recipe/1` returning 200. Our web apps are single-page apps, and a
   Pages site usually answers any path with the same page, so that check
   probably passes even when recipe 1 is broken (a guess: the sandbox
   couldn't reach the live site to confirm). bear_down's checks are
   stronger: they look for words in the live data file. Green today mostly
   means "up", not "works".
2. **Coverage is 4 projects.** The top-priority product has no requirements,
   so the process can't tell us anything about it.
3. **The loop isn't closed.** Until `eval-triage` is scheduled, a red light
   creates no work; someone has to notice it.
4. **Nobody can see the lights yet.** Verdicts are a JSON file on a branch.
   Pane's Requirements view is phase 5.
5. **Never seen a failure.** With zero fails, we haven't tested that a real
   regression goes red, files a `FIX` and goes green again.
6. **Claude cost of the agent stages.** The money cost is $0, but the
   planned daily triage and weekly judge run on Claude Code, the one paid
   service ($25 a month budget). A daily agent session that usually finds
   nothing could use a real share of that budget (a guess, since no run
   has been measured).
7. **A large plan for a small team.** Phases 3 to 8 hold 18 issues,
   including a feedback toolbar for every app and game. At the current pace
   of agent work that is weeks of Claude effort before the basics above are
   solid.

## Costs

| Piece | Money | Free-tier use | Claude effort |
| --- | --- | --- | --- |
| CI and deploy evals (GitHub Actions) | $0 | Public repo, unlimited minutes; a few minutes a night | Already built |
| Results branch | $0 | Under 1 MB a day; 1,780 lines so far | None to run |
| Live `http` checks | $0 | About 30 requests a night to our own Pages sites | None to run |
| Daily `eval-triage` (planned) | $0 | One Claude scheduled task | Uses the $25 a month Claude budget every day it runs |
| Weekly judge (planned) | $0 | One Claude scheduled task | Same, weekly |
| Chaos (planned) | $0 | Capped at 500 Worker requests a night, 0.5% of the free 100,000 a day | Building it: large |

## Recommendations

| # | Recommendation | Why | Effort |
| --- | --- | --- | --- |
| 1 | Make production checks prove the product works: run each web app's Playwright smoke tests against the live site (deploy feedback phase 3), and make `http` checks look for real content | Turns "up" into "works" | M |
| 2 | Draft requirements for Blue Eel next, then `controlled_chaos` and the CLIs | Points the process at what matters most | S each |
| 3 | Merge #772 and #771, but run triage only when something changed: a free GitHub workflow checks `verdicts.json` and wakes the agent only for a new red | Closes the loop without a daily Claude bill | S |
| 4 | Break something on purpose once: a short fire drill on one project to prove red, `FIX` and back to green | Proves the loop end to end | S |
| 5 | Bring a simple Requirements page to pane early (read-only), ahead of ratings | Makes the lights visible | M |
| 6 | Make "every must has a production check" an error, and count only checks that test content | Stops thin coverage from looking green | S |
| 7 | Pause phases 6 (in-app toolbar) and 8 (process evals) until 1 to 5 are done | Spends Claude effort where it pays off first | none |

## Decisions for Jim

1. **Triage cadence.** (a) Daily Claude task as planned in #772: $0 money,
   uses Claude budget every day. (b) Recommended: a free GitHub workflow
   runs daily and starts the agent only when a requirement turns red: $0
   money, Claude budget only when there is work. (c) Weekly only: cheapest,
   slowest.
2. **What comes next.** (a) Recommended: depth and coverage first
   (recommendations 1, 2, 4, 5). (b) Follow the plan's order (metrics,
   then chaos, then pane). Both $0 money; (a) is less Claude effort before
   it pays off.
3. **Pause the in-app feedback toolbar (phase 6)?** Recommended yes for
   now: it is the largest phase (8 issues, a guess of several weeks of
   Claude effort) and pane ratings cover most of its value.
4. **Approve requirement PRs as they come**, starting with BEAR-010 (#812)
   and, if you agree, a Blue Eel draft.

## Method

Read the plan, decision 0003, `standards/docs/evals.md`, the `evals` project
README and code, the four projects' `requirements.md` and `evals.toml`, the
workflows that run evals, the eval engine issue files, and the open PRs.
Counted results and verdicts from the `evals-results` branch at commit
`d72e8e7` (`verdicts.json` generated 2026-11-27 13:26 UTC). No product code
was changed.

## Jim's answers (2026-11-27)

1. **Triage cadence:** yes to the free workflow that files issues only when
   something needs work
   ([issue](../../../issues/eval-triage-free-workflow.md)).
2. **What comes next:** "The eval process is a continuous improvement
   process and should not be considered a one off project. The evals
   should always be green and a system of feedback to our todo process
   should be the way in which individual projects improve their eval
   coverage. Maybe we introduce an eval coverage metric / badge." Now a
   rule in `AGENTS.md` and "Continuous improvement" in
   [evals.md](../../../standards/docs/evals.md#continuous-improvement),
   with issues for the coverage metric and badge, content checks in
   production, a fire drill, Playwright smoke tests, and requirements for
   Blue Eel, controlled_chaos, homer and my_lang.
3. **In-app feedback toolbar:** "Why would I pause this? I want this." It
   stays in the plan as phase 6. The pause was suggested only to save
   Claude effort, since it is the largest phase and pane ratings overlap
   with part of it; Jim wants it, so it is not paused.
4. **Requirement PRs:** approve as they come (BEAR-010, #812, merged).
