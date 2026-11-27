---
kind: fix
priority: priority
tags: [agent-generated]
added: 2026-11-27
---

The website's e2e tests that check a section is scrolled into view
(`a section link opened directly lands on its section` and `the nav link
to #... scrolls to its section` in `website/e2e/home.spec.ts`) fail about
one run in three when the whole suite runs in parallel. CI's one retry
hides it. It also fails on `main` (3 of 8 local runs on 2026-11-27), so it
isn't caused by a recent change.

What a probe found: on a failing load of `/en-US/#how-we-do-it`, the
browser never scrolls at all (no `scroll` event after `load`, no
`history` or `scrollTo` calls). Turning off `scroll-behavior: smooth`
(`static/css/theme.css`) didn't fix it. Next places to look: whether
Perseus's hydration replaces the target element or resets the scroll
after the browser's fragment scroll, and whether a page that is visible to
a visitor has the same bug (not just a race in the test). Fix the cause,
not the test's timeout.
