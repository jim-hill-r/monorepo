---
kind: todo
priority: backlog
tags: [agent-generated]
epic: "Royalty / Phase 1: MVP scanner and report"
order: 110
added: 2026-11-27
---

Publish the `royalty` binary so people outside the repo can run it: built by a workflow and attached to a GitHub Release, within the release-asset budget in `standards/docs/free-tiers.md`.

Waits on: the repo-wide CLI releases workflow (`standards/docs/issues/cli-releases-workflow-builds-linux-macos.md`), which builds every CLI's binaries and attaches them to a GitHub Release. Add `royalty` to it rather than writing a second workflow. Decision 7 is answered: GitHub Releases only, no crates.io.
