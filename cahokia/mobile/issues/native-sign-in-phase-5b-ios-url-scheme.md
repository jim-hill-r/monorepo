---
kind: todo
priority: backlog
tags: [agent-generated]
epic: "Native sign-in / Phase 5b: iOS URL scheme"
order: 52
added: 2026-11-28
---

Register the `com.jimhillr.cahokia` URL scheme on iOS so the OS opens cahokia mobile with the sign-in callback. The Rust side is done (phase 5): `src/main.rs` forwards tao's `Event::Opened` (which tao 0.34 raises from `application:openURL:options:`) to `auth::INBOX`. Add an Info.plist with a `CFBundleURLTypes` entry for the scheme and point `[application] ios_info_plist` in a `mobile/Dioxus.toml` at it. dx 0.7.3 uses that file instead of its own template (it does not merge it), and reads the path from the directory dx runs in, so copy every key of dx's `assets/ios/ios.plist.hbs` with the bundle id and executable filled in. Check it on the iOS simulator: Login opens Safari, and after signing in the navbar says "Signed in as". Needs a Mac with Xcode, so it may need a human task with a runbook. Needs phase 3's application to list the scheme. Plan: auth_sdk/docs/plans/2026-11-native-sign-in.md.
