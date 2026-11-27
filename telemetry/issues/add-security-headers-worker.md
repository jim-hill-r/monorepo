---
kind: fix
priority: backlog
tags: [agent-generated]
epic: Threat check
order: 75
added: 2026-11-27
---

The telemetry Worker (`POST /event`) has no security headers, so THR-007 warns (rule 3 in `standards/docs/security.md`). It only answers with status codes, so the headers matter less than on a site, but rule 3 covers every public web surface: set them on every response in `src/lib.rs` next to the CORS headers (a CSP of `default-src 'none'; frame-ancestors 'none'` is enough), then point `headers` at `telemetry/src/lib.rs`. Add the headers (`Content-Security-Policy` with `frame-ancestors 'none'`, `X-Content-Type-Options: nosniff`, `Referrer-Policy`, `X-Frame-Options: DENY` and `Strict-Transport-Security`), with a test that keeps them in step with the app, then set `headers` on the surface in `telemetry/docs/threat.toml`. `cookbook/web/public/_headers` and its test in `cookbook/web/src/main.rs` are the pattern to copy. Check the CSP doesn't break the site with a Playwright run before merging.
