---
kind: todo
priority: backlog
tags: [agent-generated]
added: 2026-11-26
---

Some characters in the UI text show as empty boxes (the dash in the title and between card name and category, the "1–4" range) because Bevy's default font has no glyphs for them. Use plain ASCII ("-", "1-4") in the strings in `src/game.rs`, or bundle a font that has them.
