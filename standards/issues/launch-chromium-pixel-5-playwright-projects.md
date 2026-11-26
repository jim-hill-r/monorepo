---
kind: todo
priority: backlog
tags: [agent-generated]
epic: Browser tests on a system browser (decision 0005)
order: 10
added: 2026-11-26
---

Launch the Chromium and Pixel 5 Playwright projects in `cookbook/web` and `cahokia/web` on a system browser: `launchOptions.executablePath = process.env.CHROME_PATH` when set, else `channel: 'chrome'` when `CI` is set. With `CHROME_PATH` set and `CI` unset, run only those two projects. `global-setup.ts` uses the same launch options. See `standards/docs/decisions/0005-browser-testing.md`.
