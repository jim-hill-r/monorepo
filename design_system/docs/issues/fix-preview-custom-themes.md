---
kind: fix
priority: backlog
tags: [agent-generated]
added: 2026-11-27
---

The preview README says the site has a panel "per custom app theme", but `preview/src/main.rs` builds only the 20 standard combinations, so bear_down's Bears theme isn't shown. Add a panel for each `*/design/theme.toml` combination (and a test), or drop the claim from the README. Found while writing DSGN-005 (docs/requirements.md), which leaves custom themes out until then.
