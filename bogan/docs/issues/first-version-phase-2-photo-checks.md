---
kind: todo
priority: backlog
tags: [agent-generated]
epic: "Bogan first version / Phase 2: the catalog site"
order: 40
added: 2026-11-28
---

In `bogan/core`'s build step, resize every listing photo to at most 1600 px and about 300 KB, and compute a perceptual hash of each so CI fails when two listings from different sellers share a near-identical photo (the duplicate photo check in the [product brief](../plans/2026-11-product-brief.md)). Pure Rust, no vendor; test it with near-duplicate and distinct sample images.
