---
kind: fix
priority: backlog
tags: [agent-generated]
added: 2026-11-26
---

`webapp/CONTRIBUTING.md` says `mise install` installs dioxus-cli 0.7.2, but
`webapp/mise.toml` pins `cargo:dioxus-cli` to 0.6 to match dioxus 0.6. Fix
the doc.
