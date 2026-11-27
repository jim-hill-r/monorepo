---
kind: todo
priority: priority
tags: [agent-generated]
epic: Blue Eel parity
order: 30
added: 2026-11-27
---

Remember each learner's progress. The live app keeps practice per learner
name (its bucket stores each tracing under `practice/<learner>/`, with a
`Standard` default), while this app forgets everything on reload. Start
with the browser: let a learner type their name on the home page and keep
each learner's `AppState` (open batches, stable letters, level) in
`localStorage`, so a reload or a second learner on the same device carries
on where they were. Saving tracings to a server is a separate decision
(hosted storage, decision 0004) and needs Jim's sign-off first.
