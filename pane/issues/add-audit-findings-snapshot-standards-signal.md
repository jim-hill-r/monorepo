---
kind: todo
priority: priority
tags: [agent-generated]
epic: "Review dashboard epic / Phase 4: traffic-light dashboard"
order: 152
added: 2026-11-27
---

Add per-project `audit.sh` findings to the snapshot (give the audit a machine-readable mode, such as one `error[CODE] <project>: ...` line per finding, or JSON) and pass each project's error count to `pane_model::signals::standards`, which is grey ("Audit not run") until then.
