---
kind: todo
priority: backlog
tags: [agent-generated]
epic: Feedback on every change
order: 110
added: 2026-11-26
---

Bevy UAT. Add a scripted mode to controlled_chaos (fixed timestep, seeded RNG, timed inputs, screenshots at named frames, non-zero exit on panic) and a `smoke` task in `standards/tasks/bevy-game.toml` that runs it under `xvfb-run` and records the display with `ffmpeg`.
