---
kind: todo
priority: backlog
tags: [agent-generated]
added: 2026-11-27
---

`src/android.rs`, `src/ios.rs`, `src/linux.rs`, `src/macos.rs` and `src/windows.rs` are empty and not declared in `src/lib.rs`, so they aren't compiled and only suggest platform support that doesn't exist. cahokia's desktop and mobile crates depend on `auth_sdk` but can only use the platform-free `provider` and `redirect` modules. Delete the empty files, and say in the README which platforms work today (the browser, through `web`). Native sign-in, when a project needs it, gets its own issue with a plan for the loopback or custom-scheme redirect it needs.
