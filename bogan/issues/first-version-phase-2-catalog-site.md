---
kind: todo
priority: backlog
tags: [agent-generated]
epic: "Bogan first version / Phase 2: the catalog site"
order: 30
added: 2026-11-28
---

Build `bogan/web` (Dioxus, rendered to static files at build time, design system tokens) and `bogan/cloudflare` (Pages project `jimhillr-bogan`, created through `free-tiers.sh`, served at `jimhillr-bogan.pages.dev`): home, browse by category and region with an evidence filter, listing (photos, price, chain timeline with each hop's evidence, transparency summary, Buy from the seller link), seller and Sell on Bogan pages, and a browser-side search over a JSON index the build writes. Playwright tests tagged with the requirement IDs, `cd` http evals that check page content, `docs/cost.toml` updated, and the user guide in `docs/user/README.md` updated for buyers. See the [product brief](../docs/plans/2026-11-product-brief.md).
