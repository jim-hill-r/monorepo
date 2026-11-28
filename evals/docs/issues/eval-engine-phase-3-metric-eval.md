---
kind: todo
priority: backlog
tags: [agent-generated]
epic: Eval engine
order: 120
added: 2026-11-26
---

Eval engine phase 3. Add the `metric` eval kind with sources Web Analytics (GraphQL: LCP, INP, CLS p75, page views), Analytics Engine (errors per 1,000 views, errors after deploy, panics, CLI timings, game events), PostHog (rage, dead-click and exception session shares, funnel completion) and GitHub (deploy success rate), honouring `min_samples` and excluding synthetic events. Add `daily-observe-evals.yml` (4:00 AM Pacific) and a run one hour after each production deploy from `push-main.yml`.
