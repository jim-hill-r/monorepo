---
kind: todo
priority: priority
tags: [agent-generated]
epic: "Web UI / Phase 2: score a house by hand"
order: 20
added: 2026-11-28
---

Ship `homer/web`, a Dioxus web app (as in cookbook and cahokia, styled with `design_system`) deployed by direct upload as the Pages project `jimhillr-homer` (on `homer.jimhillr.com` once decision 0031 lands). A person sets their preferences (min and max bedrooms, min bathrooms, max price, min square feet), types a house's facts, and sees the same match score and missed criteria as the CLI, computed in the browser by `homer_core`. Preferences are kept in `localStorage` and can be exported and imported as JSON. No accounts, no backend, no analytics beyond what decision 0020 allows. Ship it with: requirements for the web journeys in `docs/requirements.md` (Jim reviews new musts) and evals (a Playwright test plus a production eval of the live page), the web user guide section in `docs/user/README.md`, `docs/cost.toml` and `docs/threat.toml` updated for a public static site, `_headers` with a strict CSP, its `standards/resources.toml` entry and its `standards/web.toml` baseline. Needs [web-ui-core-crate](web-ui-core-crate.md). See the [roadmap](../docs/roadmap.md).
