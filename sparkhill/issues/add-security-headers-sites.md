---
kind: fix
priority: backlog
tags: [agent-generated]
epic: Threat check
order: 75
added: 2026-11-27
---

Neither sparkhill site (https://jimhillr-sparkhill.pages.dev and the Blue Eel web app at https://jimhillr-sparkhill-webapp.pages.dev) has no security headers, so THR-007 warns (rule 3 in `standards/docs/security.md`). Add a `_headers` file to each Pages output (`website/dist/exported` and `webapp/dist/public`), one surface each. Add the headers (`Content-Security-Policy` with `frame-ancestors 'none'`, `X-Content-Type-Options: nosniff`, `Referrer-Policy`, `X-Frame-Options: DENY` and `Strict-Transport-Security`), with a test that keeps them in step with the app, then set `headers` on the surface in `sparkhill/docs/threat.toml`. `cookbook/web/public/_headers` and its test in `cookbook/web/src/main.rs` are the pattern to copy. Check the CSP doesn't break the site with a Playwright run before merging.
