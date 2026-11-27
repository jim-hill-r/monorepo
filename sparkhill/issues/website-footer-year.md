---
kind: fix
priority: backlog
tags: [agent-generated]
added: 2026-11-27
---

The website's footer says "© 2023 Spark Hill" in `website/src/templates/index.rs`.
Either confirm with Jim that 2023 is meant, or show the range up to the year
the site was built (the export runs at build time, so the engine can put the
current year in).
