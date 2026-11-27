---
kind: todo
priority: priority
tags: [agent-generated]
epic: "Review dashboard epic / Phase 4: traffic-light dashboard"
order: 154
added: 2026-11-27
---

Compute the live project signals (Build, Deploy and UAT, Pull requests) from the `/api/github/*` data (order 120) with the rules in `pane_model::health`, and add them to `pane_model::signals::project_signals`. Mark a `FIX` filed by a failed production smoke run (`Fix::from_smoke`, red), which `signals::open_issues` leaves false for now.
