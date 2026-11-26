---
kind: todo
priority: backlog
tags: [agent-generated]
epic: Design system
order: 40
added: 2026-11-26
---

Design system phase 1. Add `design_system/dioxus` for Dioxus 0.7: `Stylesheets {}` (embeds `dist/` with `include_str!`), `ThemePicker` and `use_theme()` writing `data-theme`, `data-scheme` and `data-a11y` on `<html>` and `localStorage`, plus the inline head script that applies the saved choice before first paint. Self-host the OFL pixel font and Atkinson Hyperlegible in `design_system/fonts/`.
