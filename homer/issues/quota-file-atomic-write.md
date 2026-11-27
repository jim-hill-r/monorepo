---
kind: fix
priority: priority
tags: [agent-generated]
added: 2026-11-26
---

`quota::save` (`../src/quota.rs`) writes the quota file in place with `fs::write`, which truncates it first. If homer is interrupted mid-write (Ctrl-C, a full disk), the file is left empty or partial, `quota::load` then reports it as corrupt, and homer refuses every lookup until the user deletes it by hand. Write to a temporary file in the same folder and `fs::rename` it over the old one, so the file is always either the old state or the new one. Add a unit test that a leftover temporary file doesn't affect `load`, and that `save` replaces an existing file.
