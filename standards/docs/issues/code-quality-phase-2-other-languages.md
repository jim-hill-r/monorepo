---
kind: todo
priority: backlog
tags: [agent-generated]
epic: "Code quality / Phase 2: Other languages"
order: 40
added: 2026-11-27
---

Add pinned mise tools and shared-task steps for Biome (`biome ci` on TypeScript and CSS, plus `tsc --noEmit`), ShellCheck and `shfmt -d` on tracked `.sh` files, actionlint in `standards/scripts/workflows-check.sh`, and `taplo fmt --check`. Warnings for one week, then failing. Add each to `standards/radar/radar.json`. What happens to the shell scripts follows Jim's answer to Question 2 in the plan. Plan: [standards/docs/plans/2026-11-code-quality.md](../plans/2026-11-code-quality.md).
