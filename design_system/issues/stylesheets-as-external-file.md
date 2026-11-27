---
kind: todo
priority: backlog
tags: [agent-generated]
epic: Cahokia strict style CSP
order: 10
added: 2026-11-27
---

Let `design_system_dioxus::Stylesheets` serve its CSS as a stylesheet file
instead of an inline `<style>`. It now puts the font faces, the tokens, the
app's theme CSS and the base sheet in one `document::Style`, so every app
that uses it (cahokia, cookbook) needs `style-src 'unsafe-inline'` in its
Content Security Policy. Pages are static on Cloudflare Pages, so a per-request
nonce isn't possible, and a hash would change on every build because the
font URLs carry asset hashes. Instead, add an option (or make it the default)
that links the tokens and base sheet as `asset!` files with
`document::Link`, and writes the `@font-face` rules into a generated CSS
asset, keeping `app_css` working. Keep the server-side render (non-wasm)
output the same for the SSR tests, and add a test that the wasm path emits no
`<style>` element.
