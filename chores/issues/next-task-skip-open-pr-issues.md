---
kind: todo
priority: priority
tags: [agent-generated]
added: 2026-11-28
---

Make `chores next-task` skip issue files that an open PR already deletes, so parallel threads stop taking work another PR has done. On 2026-11-28 three of five burst picks (code-quality phase 1 standard, resource-cleanup phase 3, review-roles phase 1) were already deleted by open PRs 948, 1005 and 972. Read the open PRs through `gh` or the API when a token is available, list the `issues/**/*.md` files each deletes (ignoring a PR that only moves them), and skip those paths; with no token, keep today's behaviour and print a note. Add a `--no-open-prs` flag for offline runs and tests.
