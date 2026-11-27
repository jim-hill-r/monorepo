---
kind: fix
priority: priority
tags: [agent-generated]
added: 2026-11-26
---

Finishing a level never leads anywhere. The "Go" button (`on_go` in
`webapp/src/components/practice.rs`) restarts the same level, so a learner
who masters the letters is never taken to the words, and the
`/congratulations` page is unreachable. After letters, "Go" should open the
word level; after words, the congratulations page.

A level also never ends on its own: once every batch is open,
`activate_expressions` in `webapp/src/state.rs` moves all stable letters
back into the active queue, and each one stabilizes again on its next
success, so `is_complete` never turns true. Decide when a level is done (for
example, once every letter is stable) and test it.
