---
kind: todo
priority: backlog
tags: [agent-generated]
epic: Design system
order: 95
added: 2026-11-26
---

Design system phase 3, the rest of cahokia. The family tree drawing in `cahokia/ui/src/family_tree.rs` still hard-codes its SVG colours (`BACKGROUND`, `LINE_COLOUR`, the text colours and the three box fills), so it stays dark brown in every theme. Use the `--ds-` tokens instead (for example `fill: var(--ds-color-surface)` and `stroke: var(--ds-color-border-strong)` through classes in `family_tree_page.css`, and `data-1` to `data-3` or surfaces for the boxes), and add `/family-tree` axe coverage in a light theme. Left out of the first cahokia move so it didn't collide with the pan and zoom work in the same file.
