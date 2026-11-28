---
kind: todo
priority: backlog
tags: [agent-generated]
epic: Browser tests on a system browser (decision 0005)
order: 20
added: 2026-11-26
---

In the `e2e` task of `standards/tasks/dioxus-web.toml` (added by PR #613), when `CI` is unset, find a Chrome (`CHROME_PATH`, `google-chrome`, `chromium`), export `CHROME_PATH` and `PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD=1` and run the tests instead of skipping; in CI install only `npx playwright install --with-deps firefox webkit`. Add each case to `standards/tests/e2e-task.test.sh`, and update `standards/docs/testing.md`, `toolchain.md` and `git-hooks.md` to say which browsers run where. Decision 0005.
