---
kind: todo
priority: backlog
tags: [agent-generated]
epic: "Native sign-in / Phase 5c: Android intent filter"
order: 54
added: 2026-11-28
---

Pass the sign-in callback to cahokia mobile on Android. tao 0.34 raises `Event::Opened` on iOS and macOS only; on Android nothing reads the intent the app is opened with, so the callback never reaches `auth::INBOX`. dx 0.7.3 lets the crate replace both files it generates: `[application] android_manifest` (add an `intent-filter` with `VIEW`, `DEFAULT`, `BROWSABLE` and `android:scheme="com.jimhillr.cahokia"`, and `launchMode="singleTask"` so the running app gets the intent) and `android_main_activity` (override `onNewIntent` and check `intent.data` in `onCreate`, then call a native function with the URL). The native side is a JNI export (`Java_dev_dioxus_main_MainActivity_onOpenUrl`) that calls `auth::INBOX.deliver`; it needs `#[unsafe(no_mangle)]`, which the workspace's `unsafe_code = "forbid"` refuses, so put it in a small crate that allows only that, or ask whether a crate such as `jni` with a safe export macro is acceptable. Check on the emulator after the Android build job exists. Also check that `webbrowser::open` opens Chrome there (it needs the activity's `ndk-context`). Plan: auth_sdk/docs/plans/2026-11-native-sign-in.md.
