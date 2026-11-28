---
kind: todo
priority: backlog
tags: [agent-generated]
epic: "Web UI / Phase 3: look up with your own key"
order: 40
added: 2026-11-28
---

Let a person look a house up by address with their own free RapidAPI key, as [decision 0001](../docs/decisions/0001-web-ui-data-source.md) describes: the key is held in `sessionStorage`, or in `localStorage` only when they tick "remember on this device"; it is sent only to RapidAPI (or the pass-through Function, if [web-ui-cors-check](web-ui-cors-check.md) found one is needed) and never logged. The browser keeps the quota state the CLI keeps and applies the same `homer_core` rules: warn at 10% left, refuse once used up until the reset. Every listing shows "Data from Zillow via RapidAPI" and links to Zillow; nothing is stored on a server. If a Function is needed it forwards only to `zillow-com1.p.rapidapi.com`, stores and logs nothing, and is rate limited per IP; add it to `docs/threat.toml` and `docs/cost.toml`. Update the user guide with how to get a free key, and add requirements and evals against a mock of the API. See the [roadmap](../docs/roadmap.md).
