---
kind: todo
priority: backlog
tags: [agent-generated]
epic: "Royalty / Phase 3: metered"
order: 20
added: 2026-11-27
---

Add `royalty sample-logs`: read a sample of logs the consumer already keeps (start with one plain access-log format), estimate `call` units with `sample_one_in`, and write the counts file. It runs as a separate process, off the serving path, and reads only what the consumer points it at.
