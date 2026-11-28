---
kind: fix
priority: backlog
tags: [agent-generated]
added: 2026-11-27
---

Opt sparkhill/webapp into the sandbox Chromium fallback (step 5 of "Cloud sandboxes" in standards/docs/git-hooks.md) by reading PLAYWRIGHT_CHROMIUM_PATH in playwright.config.ts and global-setup.ts, as pane/web does. It was left out because in a cloud sandbox e2e/install.spec.ts (SPRK-007) fails: dx serve there (0.6.3 and 0.7.3 tried, clean target/dx) serves public/ at the site root, so /assets/manifest.webmanifest returns index.html, while PR CI's dx serve passes the same test. Find why the two differ, fix it, then opt in so agents can run Blue Eel's Playwright tests before pushing.
