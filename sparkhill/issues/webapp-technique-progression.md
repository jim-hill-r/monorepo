---
kind: todo
priority: priority
tags: [agent-generated]
epic: "Blue Eel parity / Phase 5: Techniques"
order: 55
added: 2026-11-27
---

The live app (jimhillr/BlueSteel, `completedTechnique` in `frontend/src/client/src/store/common/actions.js`) moves a learner on when they finish a level with Tracing: "Go" restarts the same level with Pattern. Its next step (Pattern to Freeform) never runs because of a bug (`this.level` is undefined), so after Pattern it goes to the congratulations page. `webapp` has the three techniques (`src/technique.rs`) but "Go" always opens the next level. Decide with SPRK-004 and SPRK-009 in [requirements](../docs/requirements.md) how the technique steps fit with letters, words and capitals (for example each level Tracing, then Pattern, then Freeform), then build it with unit and Playwright tests. See "Blue Eel parity" in [README.md](README.md).
