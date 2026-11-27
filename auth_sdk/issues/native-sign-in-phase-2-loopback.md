---
kind: todo
priority: backlog
tags: [agent-generated]
epic: "Native sign-in / Phase 2: loopback for desktop"
order: 20
added: 2026-11-27
---

Add `native::loopback` (not on mobile targets): bind `127.0.0.1` on the first free port of a fixed list in the config (Auth0 has no wildcard ports), open `LoginRequest::authorize_url` in the system browser with the `webbrowser` crate, accept exactly one request to `/callback` within a timeout, answer it with a short "you can close this tab" page, and pass its URL to `NativeAuthClient::complete_login`. Test the listener natively: a request to the wrong path gets 404 and keeps waiting, the timeout returns an error, and the port list falls back when the first port is taken. Plan: auth_sdk/docs/plans/2026-11-native-sign-in.md.
