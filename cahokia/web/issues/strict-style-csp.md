---
kind: todo
priority: backlog
tags: [agent-generated]
epic: Cahokia strict style CSP
order: 30
added: 2026-11-26
---

Drop `'unsafe-inline'` from `style-src` in the web app's Content Security
Policy, in both `CONTENT_SECURITY_POLICY` in `src/main.rs` and
`public/_headers` (`tests/headers.rs` keeps them in step). Needs the family
tree's inline styles as classes first
(`ui/issues/family-tree-styles-to-css-classes.md`). The design system step
is done: in the browser `design_system_dioxus::Stylesheets` links its CSS as
files and adds no `<style>` element (see "Using it" in
`design_system/README.md`), so cahokia needs no change for it. A nonce isn't possible
on static Cloudflare Pages, which is why the plan is no inline styles at all.
Add a Playwright test that loads every page and fails on any
`securitypolicyviolation` event, so a new inline style is caught.
