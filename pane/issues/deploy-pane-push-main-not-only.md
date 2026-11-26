---
kind: todo
priority: priority
tags: [agent-generated]
epic: "Review dashboard epic / Phase 1: read-only from the repo"
order: 60
added: 2026-11-26
---

Deploy pane on every push to `main`, not only pushes under `pane/`, since its snapshot reads the whole repo. Do it in `push-main.yml` or `mise-projects.yml` without changing how other projects are picked.
