---
kind: todo
priority: backlog
tags: [agent-generated]
added: 2026-11-28
---

cahokia desktop (`desktop/src/auth.rs`, PR 965) and mobile (`mobile/src/auth.rs`) each have their own `AuthView`, `AuthControls`, `display_name` and `error_message`, which differ only in mobile's Cancel button and desktop's loopback port message. Once both are on `main`, move the shared parts into the `ui` crate (it already holds what every platform shows) and keep only the platform sign-in (loopback or URL scheme) in each crate.
