---
kind: todo
priority: backlog
tags: [agent-generated]
epic: Feedback on every change
order: 160
added: 2026-11-26
---

Add PostHog session replay to the other production web apps: pane, cahokia and bear_down (production host only, inputs masked, console and network timing on), with a privacy note on each site. Copy cookbook's pattern: `cookbook/web/public/posthog.js` loaded from `index.html` or `Dioxus.toml`, the two PostHog hosts in the CSP (both `_headers` and any `<meta>` CSP), a privacy note, a `PostHog` service in `docs/cost.toml` and an information-disclosure threat in `docs/threat.toml`. Leave out sparkhill (Blue Eel and its site stay free of third-party scripts, SPRK-006) and controlled_chaos (game events, not replay). First check cookbook's recordings show up in PostHog, so a broken loader isn't copied. The snippet must not load when the user agent contains `jimhillr-evals` (every Playwright, smoke and eval run sends it, eval engine phase 3), and a Playwright test should check that.
