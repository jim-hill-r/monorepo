---
kind: todo
priority: backlog
tags: [agent-generated]
epic: "Web standard / Phase 2: Lighthouse CI"
order: 20
added: 2026-11-28
---

Add a workflow that runs @lhci/cli 0.16.1 (pinned) on the runner's Google Chrome (CHROME_PATH, no download) for each site a PR changes, against its build output (staticDistDir), and after each deploy against the live URL. Add a lighthouserc.json per site with the WEB-1xx to WEB-3xx assertions as warnings (WEB-404), upload.target filesystem, reports as a 14-day artifact, and record each site's measured scores and sizes as its baseline in standards/web.toml (WEB-403). See [web.md](../web.md#lighthouse-ci) and [decision 0028](../decisions/0028-web-checks.md). Update the standards project's docs/cost.toml for the runner minutes.
