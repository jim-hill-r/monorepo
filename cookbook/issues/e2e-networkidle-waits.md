---
kind: fix
priority: backlog
tags: [agent-generated]
added: 2026-11-27
---

The Playwright tests in `web/tests/` wait for `networkidle` about 100 times. Playwright discourages it, and on CI it can outlast the test timeout while the Auth0 discovery request is in flight (it timed out `auth-navbar.spec.ts` "login button on all pages" on Mobile Chrome in PR 747, which now waits for the header instead). Replace the rest with web-first assertions on the element each test needs.
