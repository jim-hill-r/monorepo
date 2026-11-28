---
kind: todo
priority: backlog
tags: [agent-generated]
added: 2026-11-28
---

Cloudflare Web Analytics sites exist for bear_down, cahokia, controlled_chaos, cookbook, design_system and sparkhill (`infra/src/stacks/cloudflare.rs`), but with `auto_install = false`, because Cloudflare refuses auto-install on a bare `jimhillr-*.pages.dev` host. Until each app loads the beacon itself, the sites record nothing.

Add `<script defer src="https://static.cloudflareinsights.com/beacon.min.js" data-cf-beacon='{"token": "<site token>"}'></script>` to each of those apps. The site token is public (it ships in every page), so it may live in the repo: read it from the applied `cloudflare` stack (expose `site_token` per site as an OpenTofu output, and have a workflow, not a person, write the tokens to a committed file such as `infra/web-analytics.toml` that each app build reads). Check IAC-002 doesn't flag the token as sensitive. The apps' CSPs already allow `static.cloudflareinsights.com` and `cloudflareinsights.com`, except cookbook and cahokia (`issues/web-analytics-beacon-blocked-by-csp.md`, which this can absorb). Add an eval that each production page carries the beacon, and update `infra/README.md`. Split per app if it grows past one PR.
