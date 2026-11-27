---
kind: todo
priority: backlog
tags: [agent-generated]
epic: Feedback on every change
order: 10
added: 2026-11-26
---

Preview deploys. Add a `preview` task to `standards/tasks/cloudflare-pages.toml` (`wrangler pages deploy --branch pr-<n>`), run it in PR CI for affected deploy projects on same-repo PRs, and keep one sticky PR comment with the preview URLs.

Add the same `preview` task to `standards/tasks/bevy-game.toml` for games with a `wrangler.toml` (today controlled_chaos): `mise run release:web`, then `wrangler pages deploy dist/web --branch pr-<n>`, so the comment has a playable link. The web build compiles for `wasm32-unknown-unknown` (checked 2026-11-27; about 10 minutes cold, `game_bg.wasm` about 17 MB, under the 25 MiB Pages file limit), so cache `target/` to keep the Actions minutes down.
