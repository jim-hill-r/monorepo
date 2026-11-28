---
kind: todo
priority: backlog
tags: [agent-generated]
epic: "Web UI / Phase 3: look up with your own key"
order: 30
added: 2026-11-28
---

Find out whether `zillow-com1.p.rapidapi.com` answers a browser `fetch` from another origin: does it answer the CORS preflight for the `x-rapidapi-key` and `x-rapidapi-host` headers, and does it expose the `x-ratelimit-requests-*` headers to scripts (`Access-Control-Expose-Headers`)? Check from a GitHub Actions run or a local Playwright page with a real request and a throwaway free key the person running it owns, never Jim's key from a workflow (FT-002). Record the answer in [decision 0001](../docs/decisions/0001-web-ui-data-source.md): direct browser calls, or the pass-through Pages Function the decision describes. See the [roadmap](../docs/roadmap.md).
