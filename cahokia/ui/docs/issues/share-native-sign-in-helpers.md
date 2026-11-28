---
kind: todo
priority: backlog
tags: [agent-generated]
added: 2026-11-28
---

The desktop and mobile apps each have their own copy of the native sign-in helpers: `AUTH0_ISSUER`, `NATIVE_CLIENT_ID`, `provider_config`, `display_name` and the short `error_message` text (`cahokia/desktop/src/auth.rs` and `cahokia/mobile/src/auth.rs`), so a wording or tenant change has to be made twice. Move them into `ui` (or a small `auth` module there) with their tests, keep each platform's flow (loopback on desktop, custom scheme on mobile) in its own crate, and make phase 3's client ID a single constant. Plan: auth_sdk/docs/plans/2026-11-native-sign-in.md.
