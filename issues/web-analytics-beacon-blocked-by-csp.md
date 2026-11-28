---
kind: fix
priority: backlog
tags: [agent-generated]
added: 2026-11-27
---

Cloudflare Web Analytics can't report on `cookbook` and `cahokia`. infra turns it on for both (`web_apps()` in `infra/src/stacks/mod.rs`), so Pages injects its beacon (a script from `https://static.cloudflareinsights.com` that posts to `https://cloudflareinsights.com`), but both apps' CSP (the `<meta>` CSP in `web/src/main.rs` and the same CSP in `web/public/_headers`, kept equal by tests) allows neither origin. Add `https://static.cloudflareinsights.com` to `script-src` and `https://cloudflareinsights.com` to `connect-src` in both places, as the other sites' `_headers` do. Left out of the security headers PR because both `main.rs` files were in open PRs (#780, #783) at the time.
