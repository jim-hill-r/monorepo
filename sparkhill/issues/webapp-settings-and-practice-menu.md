---
kind: todo
priority: priority
tags: [agent-generated]
epic: "Blue Eel parity / Phase 6: Settings and menu"
order: 60
added: 2026-11-27
---

The live app (blue.eel.education, checked 2026-11-27) has a left menu that opens Letter Practice (`/letter`) or Word Practice (`/word`) directly, and a settings panel behind the top-right button with Student (the learner's name), Sequence (the letters or words to practise) and Technique. `webapp` instead asks for a name on the home page and always goes letters, words, capitals. Add the menu and a settings panel with Student and Sequence (Technique comes from webapp-pattern-freeform-techniques), keep the home page's Begin, and update the Playwright tests. Leave out the live app's Upload Practice switch: it sends learners' tracings to S3, which SPRK-006 rules out ([requirements](../docs/requirements.md)). See "Blue Eel parity" in [README.md](README.md).
