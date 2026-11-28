---
kind: todo
priority: backlog
tags: [agent-generated]
epic: "Native sign-in / Phase 5b: Android callback"
order: 55
added: 2026-11-28
---

Let Android finish the mobile sign-in that iOS does in `src/auth.rs`. The spike for phase 5 found that tao 0.34 (under Dioxus 0.7) emits `Event::Opened` for a custom URL scheme on iOS and macOS but never on Android: it doesn't forward the activity's intent, and the `MainActivity.kt` dx writes has no `onNewIntent`. So: add an `intent-filter` for `com.jimhillr.cahokia://callback` (`VIEW`, `DEFAULT`, `BROWSABLE`) to a custom manifest through `android_manifest` in `Dioxus.toml`, set `android:launchMode="singleTask"` so the browser returns to the running app, and hand the intent's data URL to Rust through a custom `android_main_activity` (for example by evaluating a small script in the WryActivity web view that Dioxus reads with `document::eval`, since the workspace forbids the `unsafe` a JNI export would need). Then drop `unsupported_reason` for Android. Check first whether a newer tao forwards Android intents, which would make this a version bump (dioxus is pinned: don't bump it for this). Plan: auth_sdk/docs/plans/2026-11-native-sign-in.md.
