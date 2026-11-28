---
kind: todo
priority: priority
tags: [agent-generated]
epic: "Review dashboard epic / Phase 7: revenue and cost"
order: 340
added: 2026-11-26
---

Add `/api/free-tiers` to the pane Worker and show it as a headroom table on the Money page. The data is ready: every All Free Tier Usage run uploads `free-tier-usage.json` as its `free-tier-usage` artifact (`{version, taken_at, over, items: [{item, used, warn_at, limit, status, note}]}`, `used` null when skipped; see `standards/scripts/free-tiers.sh`). The route downloads the latest run's artifact and caches it for an hour. It needs the Worker's GitHub token, so do it after the Phase 3 GitHub routes (order 120).
