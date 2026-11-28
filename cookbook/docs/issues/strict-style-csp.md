---
kind: todo
priority: backlog
tags: [agent-generated]
added: 2026-11-27
---

Drop `'unsafe-inline'` from `style-src` in the web app's Content Security
Policy, in both `CONTENT_SECURITY_POLICY` in `web/src/main.rs` and
`web/public/_headers`. In the browser, `design_system_dioxus::Stylesheets`
now links its CSS as files instead of an inline `<style>` (see "Using it" in
`design_system/README.md`), and the web app sets no `style` attributes, so
nothing should need it any more. Load every page in the Playwright tests and
fail on any `securitypolicyviolation` event, so a new inline style is caught.
