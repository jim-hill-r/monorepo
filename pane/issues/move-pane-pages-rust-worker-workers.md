---
kind: todo
priority: priority
tags: [agent-generated]
epic: "Review dashboard epic / Phase 3: live GitHub data"
order: 100
added: 2026-11-26
---

Move pane from the Pages project to a Rust Worker (workers-rs) that serves the Dioxus build as Workers static assets with `run_worker_first` for `/api/*` only. Add or extend a shared task file for Workers-with-assets projects (bear_down needs the same), update `DEPLOY_URL`, and add a human issue file to delete the old `pane` Pages project once the Worker is live.
