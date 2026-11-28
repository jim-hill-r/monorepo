---
kind: todo
priority: backlog
tags: [agent-generated]
epic: "Native sign-in / Phase 4: desktop login"
order: 40
added: 2026-11-27
---

Replace the Login `TODO` in `src/main.rs`: on click, run `auth_sdk::native::loopback::sign_in` (phase 2, done) with the native Auth0 application from phase 3, then show the user's name and a Sign out button in the navbar, and the `AuthError` as a short message when it fails. Keep the session in memory. Test the navbar states with `dioxus-ssr` and the sign-in against a fake provider. Needs phases 2 and 3. Plan: auth_sdk/docs/plans/2026-11-native-sign-in.md.
