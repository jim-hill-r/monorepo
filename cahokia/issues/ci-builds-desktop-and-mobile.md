---
kind: todo
priority: backlog
tags: [agent-generated]
added: 2026-11-26
---

CI never builds or tests the desktop and mobile apps. `cahokia/Cargo.toml`
sets `default-members = ["core", "ui"]`, so `mise run ci` skips them, and they
need GTK and WebKit system packages (`libwebkit2gtk-4.1-dev`, `libgtk-3-dev`,
`libxdo-dev`) to compile. Add those to cahokia's `ci:deps` task and run
`cargo test -p desktop -p mobile` in CI, so their route tests catch breakage.
