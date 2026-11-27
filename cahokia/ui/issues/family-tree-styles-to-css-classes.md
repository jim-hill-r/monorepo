---
kind: todo
priority: backlog
tags: [agent-generated]
epic: Cahokia strict style CSP
order: 20
added: 2026-11-27
---

Move `FamilyTreeView`'s inline `style` attributes into a stylesheet. Seven
elements in `ui/src/family_tree.rs` set `style: "…"` (the toolbar, the SVG
frame, edge paths, person groups, person boxes and both text lines). A
`style` attribute set with `setAttribute` is blocked by a CSP without
`'unsafe-inline'`. Every value comes from a fixed set (the `--ds-` token
constants, the lineage and dimmed states), so give each state a class
(several already exist, such as `on-lineage`, `dimmed`, `focus`) and put the
rules in a `family_tree.css` asset linked by the component. Keep the render
tests passing and update them to check classes instead of inline styles;
the web Playwright tests in `web/tests/family-tree.spec.ts` check the drawing
still looks the same.
