---
kind: todo
priority: backlog
tags: [agent-generated]
epic: "Native sign-in / Phase 5: mobile login"
order: 50
added: 2026-11-27
---

Replace the Login `TODO` in `src/main.rs` with sign-in through a custom URL scheme (such as `com.jimhillr.cahokia://callback`): open `NativeAuthClient::begin_login`'s URL in the system browser, register the scheme (Android intent filter, iOS URL type), and pass the URL the OS opens the app with to `complete_login`. Start with a spike into how Dioxus mobile hands an opened URL to the app; if it can't yet, record that and split the work. Needs phase 3's application to list the scheme. Plan: auth_sdk/docs/plans/2026-11-native-sign-in.md.
