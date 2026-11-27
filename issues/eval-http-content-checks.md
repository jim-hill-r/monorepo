---
kind: todo
priority: priority
tags: [agent-generated]
epic: Eval engine
order: 95
added: 2026-11-27
---

Make production `http` evals check content. Most `cd` evals only check that a path answers (cookbook `/recipe/1`, cahokia `/family-tree`, pane `/`), and a single-page app on Pages answers every path with the same page, so they pass even when the page is broken ([executive review](../docs/reviews/2026-11-27-evals/review.md), gap 1). Add `body_contains` (or a check of a JSON data file, as bear_down does) to every page `http` eval in bear_down, cahokia, cookbook and pane, and add EVL-005 to `evals check`: an `http` eval with a `path` and no body check is a warning. Confirm first with one request whether an unknown path on each site returns 200.
