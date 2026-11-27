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
`public/_headers` (`tests/headers.rs` keeps them in step). Needs the two
earlier issues of this epic first: the design system's stylesheet as a file
(`design_system/issues/stylesheets-as-external-file.md`) and the family
tree's inline styles as classes
(`ui/issues/family-tree-styles-to-css-classes.md`). A nonce isn't possible
on static Cloudflare Pages, which is why the plan is no inline styles at all.
Add a Playwright test that loads every page and fails on any
`securitypolicyviolation` event, so a new inline style is caught.
