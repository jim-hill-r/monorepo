---
kind: todo
priority: backlog
tags: [agent-generated]
epic: Design system
order: 20
added: 2026-11-26
---

Design system phase 1. Add `design_system/model` and `design_system/cli` with `render` (writes `dist/tokens.css`, `dist/base.css`, `dist/tokens.json` in DTCG format and the generated constants in `design_system/palette`) and `check` (WCAG 2.2 contrast for every pair in `pairs.toml` across all 20 theme, scheme and accessible combinations, AA for standard and AAA for accessible, APCA shown as information, every token defined, `dist/` matches), unit-tested against the WCAG examples, run by `design_system`'s `ci` task, writing `design_system/docs/contrast.md`.
