---
kind: todo
priority: backlog
tags: [agent-generated]
epic: Feedback on every change
order: 115
added: 2026-11-27
---

Bevy UAT in CI. controlled_chaos has a scripted mode and a project-level `mise run smoke` (`controlled_chaos/smoke/run.sh`: the script under `xvfb-run`, recorded with `ffmpeg`, output in `target/smoke`). Move that runner into a shared `smoke` task in `standards/tasks/bevy-game.toml` (a game opts in by having `smoke/smoke.script`), install `ffmpeg`, `libxkbcommon-x11-0` and `mesa-vulkan-drivers` in `ci:deps` (llvmpipe renders it on a runner with no GPU), run it in PR CI for changed games, and upload `target/smoke` with the UAT report (phase 3a's `uat-reports` site, or an Actions artifact until that exists).
