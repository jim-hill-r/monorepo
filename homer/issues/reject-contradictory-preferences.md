---
kind: todo
priority: priority
tags: [agent-generated]
added: 2026-11-27
---

homer accepts preferences no property can meet, such as `--min-bedrooms 4 --max-bedrooms 2`, spends a RapidAPI free-quota request, and then reports the house as failing. Add a `HomePreferences::validate` (in `src/main.rs`, called from `main` before the client is built) that fails when `min_bedrooms > max_bedrooms`, with a message naming both flags, and add a trycmd transcript in `tests/cmd/` showing the error and that no request was made.
