---
kind: todo
priority: backlog
tags: [agent-generated]
added: 2026-11-27
---

The website's tests render the home page template with Sycamore's SSR, but
nothing loads the real `perseus export` output in a browser, so a broken
hydration, a stylesheet or image path that 404s under `.perseus/static/`, or
a layout that breaks at phone width would only show on the live site. Add a
Playwright smoke test that serves `dist/exported` and checks the page, its
nav anchors and its images at desktop and phone widths.
