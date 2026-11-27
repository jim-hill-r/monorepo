---
kind: fix
priority: backlog
tags: [agent-generated]
epic: Threat check
order: 75
added: 2026-11-27
---

The design_system preview (https://jimhillr-design-system.pages.dev) has no security headers, so THR-007 warns (rule 3 in `standards/docs/security.md`). Add a `_headers` file to the preview's Pages output (`design_system/preview/dist`). Add the headers (`Content-Security-Policy` with `frame-ancestors 'none'`, `X-Content-Type-Options: nosniff`, `Referrer-Policy`, `X-Frame-Options: DENY` and `Strict-Transport-Security`), with a test that keeps them in step with the app, then set `headers` on the surface in `design_system/docs/threat.toml`. `cookbook/web/public/_headers` and its test in `cookbook/web/src/main.rs` are the pattern to copy. Check the CSP doesn't break the site with a Playwright run before merging.
