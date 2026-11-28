---
marp: true
theme: default
paginate: true
size: 16:9
title: evals executive review, 2026-11-27
style: |
  section { font-size: 22px; }
  table { font-size: 18px; }
  section.lead h1 { font-size: 52px; }
  code { font-size: 18px; }
  pre { font-size: 16px; }
---

<!-- _class: lead -->

# The eval process
## Executive review, 2026-11-27

Reviewer: Claude, at Jim's request ("I want to understand this better")

The full written review is [review.md](review.md). Figures are from the
`evals-results` branch as of 2026-11-27 13:26 UTC.

---

# Executive summary

**What it is:** a way to answer, for every product, *does what we shipped
still do what we said it should?* Promises are written down as numbered
requirements, checked automatically after every build and deploy, and shown
as one traffic light each.

**Where it stands:** the bones work after two days. 4 projects, 35
requirements, 88 evals, results on every push, 12 requirements green.

**But the green is thin:**
1. Production checks mostly prove a page *loads*, not that it *works*.
2. Blue Eel, the top priority, has no requirements yet.
3. Nothing turns a red light into work yet, and nobody can see the lights.

---

# Scorecard

| Area | Rating | Why |
| --- | --- | --- |
| Goals | Good | Clear single question, approved plan and decision |
| Design | Good | Strict verdict rules, reuses existing tests, plain files |
| Coverage | Poor | 4 projects; top-priority Blue Eel and all CLIs and games missing |
| Depth of checks | Poor | All 53 production results are single web requests, most "is it up" |
| Feedback loop | Fair | Results stored and verdicts computed; triage not live, no page to see them |
| Cost | Good | $0 money, no new vendor; agent stages will use Claude budget |
| Safety | Good | Chaos capped and off by default; live checks tagged as test traffic (Playwright not yet) |

---

# Top recommendations

| # | Do this | Effort |
| --- | --- | --- |
| 1 | Run Playwright smoke tests against the live sites, and make web checks look for real content | M |
| 2 | Write Blue Eel's requirements next, then controlled_chaos and the CLIs | S each |
| 3 | Run triage only when something turns red (free workflow wakes the agent) | S |
| 4 | Break something on purpose once, to prove red, FIX, green | S |
| 5 | A simple read-only Requirements page in pane, early | M |

---

<!-- _class: lead -->

# What it is and why

---

# The problem it solves

- **Tests** say the code does what the code says.
- They don't say the product does what *Jim* wants, live, for real users.
- Before this, "what Jim wants" lived in READMEs, plans and review decks.
  Nothing checked it after a deploy.

The eval process makes intent **written, numbered and checked**, the same
way in every project.

Approved 2026-11-26: the
[plan](../../../standards/docs/plans/2026-11-eval-engine.md) and
[decision 0003](../../../standards/docs/decisions/0003-eval-engine.md).

---

# Three words to know

| Word | Means | Example |
| --- | --- | --- |
| **Requirement** | A promise, with an ID and a priority (must, should, could) | COOK-001: there is a recipe for every day of the year (must) |
| **Eval** | One check of one requirement | Test `cook_001_...` passes; `/recipe/1` answers on the live site |
| **Verdict** | The light for a requirement | Green met, amber at risk, red not met, grey unknown |

Jim approves requirements (they define "working"). Agents draft them and
write the evals.

---

# How the light is decided

| Light | Rule (simplified) |
| --- | --- |
| Red, not met | Latest production check failed, or Jim rated it not met since the last deploy |
| Amber, at risk | Flaky (2 of last 7 failed), near a limit, stale (no production result in 3 days), or a hand-made setting never checked |
| Green, met | Every eval passes **and** a production result is under 2 days old |
| Grey, unknown | No evals, or no production result yet |

Strict on purpose: **no production evidence means grey, never green.**
Only must requirements can turn a project red.

---

<!-- _class: lead -->

# How it works, end to end

---

# The flow

```
 requirements.md      evals.toml          after CI / after deploy       evals-results branch     verdict
 COOK-001 (must) ──► recipe tests (ci) ──► read test reports ──────────► results/*.jsonl ───────► green
                     /recipe/1 (cd)        request the live site          verdicts.json             │
                                                                                                    ▼
                                                          (planned) eval-triage files a FIX ◄── red
```

- **Write** the promise, **tie** checks to it; the audit checks the files.
- **After CI:** read the test reports the build already wrote.
- **After deploy:** check the live site; a failing must turns the deploy red.
- **Store** results as files on a separate branch, and **compute** the light.
- **(Planned)** a daily agent turns red into a `FIX` issue.

---

# Stages: when evals run

| Stage | When | State |
| --- | --- | --- |
| After CI | Every PR, push and nightly build | Live |
| After deploy | Right after each production deploy | Live (web requests only) |
| Observability | Daily, against real-user speed and error data | Planned, phase 3 |
| Chaos | Nightly, faults injected in our own sessions | Planned, phase 4 |
| Jim's ratings | When Jim rates in pane | Planned, phase 5 |
| Agent judge | Weekly, for taste-type promises | Planned, phase 7 |
| Hand-made settings | When due, through the weekly digest | Live; first ask due Monday |

---

# Settings made by hand

Some things live only in dashboards: Auth0 apps, tokens, connectors,
Access policies. They break silently.

- 36 are listed in `standards/settings.toml`, each with what it must be and
  how often to check.
- The weekly human digest asks Jim about the due ones; "ok" is recorded as a
  passing result.
- None answered yet (the system is one day old). That is why cahokia's
  sign-in requirement is amber today.

---

# Where the code lives

Added 2026-11-28 at Jim's request. One project, `evals/`, holds the engine;
the rest lives next to what it checks, on purpose.

| What | Where |
| --- | --- |
| Engine: types and verdict rules; the `evals` command | `evals/model`, `evals/cli`, `evals/verdicts.toml` |
| Each project's promises and checks | `<project>/docs/requirements.md`, `<project>/evals.toml` |
| Format and rules | `standards/docs/evals.md`, plan, decision 0003 |
| CI and deploy wiring | `mise-projects.yml`, `push-main.yml`, `nightly-*.yml`, `standards/scripts/evals.sh` |
| Store, triage, fire drill | `evals-store.yml`, `evals-triage.yml`, `evals-fire-drill.yml` |
| Results and verdicts | `evals-results` branch |
| Hand-made settings | `standards/settings.toml`, `standards/settings-checks.jsonl` |
| Its to-do list | `evals/issues/` ([roadmap](../../../evals/docs/roadmap.md)) |

---

# Dependencies and vendors

| Dependency | Version | Used for |
| --- | --- | --- |
| serde, serde_json, toml | 1, 1, 1 | Reading `evals.toml`, writing results and verdicts |
| chrono | 0.4 | Result times and freshness (flagged unmaintained: a Threat check issue) |
| clap, anyhow, thiserror | 4, 1, 2 | Command line and errors |
| ureq | 3 | The live `http` evals |

| Vendor | Plan | Cost a month | Billing check | Approved |
| --- | --- | --- | --- | --- |
| GitHub Actions | Free (public repo) | $0 | Billing API plus a settings check | Yes |
| Cloudflare Pages | Free (the projects' own sites, about 30 requests a night) | $0 | Billing Read API plus a settings check | Yes |
| Claude Code | The one paid service | Share of $25 (on-demand `eval-triage` skill) | Settings check | Yes |

---

<!-- _class: lead -->

# Where it stands

---

# Progress by phase

| Phase | State |
| --- | --- |
| 0 Foundations | Done |
| 1 After CI | Done, except a PR-comment list that waits for preview deploys |
| 2 After deploy | Partly: live checks merged (#755); CLI checks (#771) and triage (#772) open |
| 3 Observability | Not started |
| 4 Chaos in production | Not started |
| 5 Ratings in pane | Not started |
| 6 In-app comment toolbar | Not started (largest: 8 issues) |
| 7 Agent judge | Not started |
| 8 Process evals | Not started |

20 eval engine issues remain, all in the backlog.

---

# Coverage by project

| Project | Requirements (musts) | Evals | Green | Amber | Grey |
| --- | --- | --- | --- | --- | --- |
| bear_down | 9 (6) | 17 | 6 | 0 | 3 |
| cahokia | 7 (2) | 25 | 1 | 1 | 5 |
| cookbook | 9 (3) | 15 | 2 | 0 | 7 |
| pane | 10 (2) | 31 | 3 | 0 | 7 |
| **Total** | **35 (13)** | **88** | **12** | **1** | **22** |

Not covered: **Blue Eel** (top priority), sparkhill website, blueeel,
controlled_chaos, design_system, telemetry, homer, my_lang.

---

# Why so much grey

- Most should and could requirements have **only CI evals**. The rules
  refuse green without production evidence, so they stay grey. Working as
  designed.
- **COOK-005 (accessibility, a must)** is grey: its production check is a
  Playwright test, and nothing runs Playwright against production yet.
- **CAHO-003 (sign-in, a must)** is amber: its Auth0 setting has never been
  checked by hand. The first digest that asks will clear it.

---

# Results so far

| | Count |
| --- | --- |
| Runs stored (since 2026-11-26 23:11 UTC) | 41 |
| Results from CI | 1,727 |
| Results from production | 53 |
| **Failures** | **0** |

Good for the products. But it also means **we have never seen the process
catch something**, go red, file a fix and go green again.

---

<!-- _class: lead -->

# What's working, and what isn't

---

# What's working

- **One question, one light per promise**, the same in every project.
- **Strict rules**: no evidence is grey, an unchecked setting is amber.
- **Reuses existing tests**: evals read the reports CI already writes, so
  nothing runs twice.
- **Plain files, no new vendor, $0**: results are in git, readable on
  GitHub, by pane and by agents.
- **Guard rails in the build**: the audit rejects evals that point at
  nothing; a failed must after deploy turns `main` red.
- **Jim holds the pen on intent**: requirements need his approval.

---

# Gap 1: "green" mostly means "up"

- All 53 production results are single web requests.
- Most only check that a page answers, such as `GET /recipe/1` returns 200.
- Our web apps are single-page apps. A Pages site usually serves the same
  page for any path, so that check probably passes even if recipe 1 is
  broken (a guess: this sandbox couldn't reach the live site to confirm).
- bear_down is the exception: its checks look for real words in the live
  data file.

**Fix:** run the Playwright smoke tests against the live site, and make
web checks look for content.

---

# Gaps 2 to 4

**2. Coverage.** Only 4 projects. The process says nothing about Blue Eel,
the games or the CLIs.

**3. The loop isn't closed.** Until triage runs, a red light creates no
work. Someone has to notice it.

**4. Nobody can see the lights.** Verdicts are a JSON file on a branch.
Pane's Requirements view is phase 5.

---

# Gaps 5 to 7

**5. Never seen a failure.** We haven't proven a real regression goes red,
files a `FIX` and goes green.

**6. Claude cost of agent stages.** Money is $0, but daily triage and the
weekly judge run on Claude Code, the one paid service ($25 a month). A daily
session that usually finds nothing could use a real share of it (a guess;
no run measured yet).

**7. A big plan.** Phases 3 to 8 hold 18 issues, including a feedback
toolbar for every app and game: weeks of Claude effort before the basics
are solid.

---

# Costs

| Piece | Money | Free-tier use | Claude effort |
| --- | --- | --- | --- |
| CI and deploy evals | $0 | A few Actions minutes a night (public repo, unlimited) | Built |
| Results branch | $0 | Under 1 MB a day | None to run |
| Live web checks | $0 | About 30 requests a night to our own Pages sites | None to run |
| Daily triage (planned) | $0 | One Claude scheduled task | Every day it runs |
| Weekly judge (planned) | $0 | One Claude scheduled task | Weekly |
| Chaos (planned) | $0 | Capped at 500 Worker requests a night (0.5% of free) | Large to build |

No new vendor, no database, nothing that bills.

---

<!-- _class: lead -->

# Recommendations and decisions

---

# Recommendations

| # | Recommendation | Impact | Effort |
| --- | --- | --- | --- |
| 1 | Playwright smoke tests against production; content checks in web evals | Green means "works" | M |
| 2 | Requirements for Blue Eel, then controlled_chaos and the CLIs | Covers what matters most | S each |
| 3 | Merge #771 and #772, gate triage on a free workflow that wakes the agent only for a new red | Loop closed, small Claude bill | S |
| 4 | One fire drill: break a check on purpose, watch red, FIX, green | Proves the loop | S |
| 5 | Read-only Requirements page in pane, ahead of ratings | Lights visible | M |
| 6 | "Every must has a production check" becomes an error, counting only content checks | No thin green | S |
| 7 | Pause phase 6 (in-app toolbar) and phase 8 (process evals) | Effort where it pays first | none |

---

# Decisions for Jim (1 of 2)

**1. How often does triage run?**

| Option | Money | Claude budget |
| --- | --- | --- |
| Daily Claude task, as in #772 | $0 | Uses budget every day |
| **Recommended:** free daily workflow, wakes the agent only on a new red | $0 | Only when there is work |
| Weekly only | $0 | Lowest; slowest to react |

**2. What comes next?**

| Option | Money | Claude budget |
| --- | --- | --- |
| **Recommended:** depth and coverage first (recommendations 1, 2, 4, 5) | $0 | Pays off soonest |
| The plan's order: metrics, then chaos, then pane | $0 | More effort before the lights are trustworthy |

---

# Decisions for Jim (2 of 2)

**3. Pause the in-app feedback toolbar (phase 6)?**
Recommended yes for now. It is the largest phase (8 issues, a guess of
several weeks of Claude effort), and ratings in pane cover most of its
value. $0 money either way.

**4. Approve requirement PRs as they come.**
BEAR-010 (#812) is waiting now; a Blue Eel draft would be next.

Reply on this PR with your picks, or in the project chat.

---

# Appendix: method and evidence

- Read: the [plan](../../../standards/docs/plans/2026-11-eval-engine.md),
  [decision 0003](../../../standards/docs/decisions/0003-eval-engine.md),
  [evals.md](../../../standards/docs/evals.md),
  [evals/README.md](../../../evals/README.md), the four projects'
  `docs/requirements.md` and `evals.toml`, `mise-projects.yml`,
  `evals-store.yml`, the `issues/eval-engine-*.md` files and the open PRs.
- Counted from the `evals-results` branch at `d72e8e7`
  (`verdicts.json` generated 2026-11-27 13:26 UTC): 41 commits, 1,780
  result lines, 0 failures.
- Synthetic traffic: the `telemetry` Worker drops tagged requests
  (`telemetry/src/event.rs:18`); live web checks send the `jimhillr-evals`
  user agent (`evals/cli/src/http.rs:17`). Playwright runs are not tagged
  yet (phase 3 issue).
- No product code changed in this review.

---

# Jim's answers, 2026-11-27

| Decision | Answer |
| --- | --- |
| 1. Triage cadence | Yes: a free workflow files issues only when there is work |
| 2. What comes next | Evals are continuous improvement, not a project: always green, red feeds the issue process, coverage grows per project, with a coverage metric and badge |
| 3. Pause the in-app toolbar? | No, Jim wants it. It stays as phase 6 |
| 4. Requirement PRs | Approve as they come (#812 merged) |

---

# Jim's feedback, 2026-11-28

"I agree with everything." What followed:

- The findings are now the [evals roadmap](../../../evals/docs/roadmap.md):
  trust the green, cover every project, deeper evidence, then chaos and
  the judge. Its issues are in `evals/issues/` and each project's own
  `issues/`.
- Every review deck now has **Where the code lives** and **Dependencies
  and vendors** slides (the `deep-project-review` skill and
  `docs/reviews/README.md`).
- The review toolbar moves to the feedback component as one tool: who is
  signed in (user, app manager, product owner, engineer) picks the kind of
  feedback and where it goes. The eval engine only reads the results.
