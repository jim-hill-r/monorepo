---
kind: todo
priority: backlog
tags: [agent-generated]
added: 2026-11-27
---

Save the display settings (`GameTheme` in `src/theme.rs`: light or dark, and accessible mode) between launches. Today they reset to gamey dark, standard, each time the game starts. The design system plan keeps such choices in files, not a server: `localStorage` on the web build and a small TOML file in the platform's config directory on desktop and Android. Also start in accessible mode when the platform asks for more contrast, where Bevy or the browser exposes it.
