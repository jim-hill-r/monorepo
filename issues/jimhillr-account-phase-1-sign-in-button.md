---
kind: todo
priority: priority
tags: [agent-generated]
epic: "jimhillr account / Phase 1: one sign-in button"
order: 10
added: 2026-11-28
---

Add `design_system_dioxus::AccountButton` (signed out: **Sign in with jimhillr** and the hint "One jimhillr account works in every jimhillr app."; signed in: the person's name and **Sign out**; an error state) taking plain values and callbacks, so the design system doesn't depend on `auth_sdk`. Move cookbook's `format_auth_error` into `auth_sdk` as `AuthError::user_message`, implement `WebAuthProvider::is_authenticated` and `logout` (closes [auth_sdk/issues/web-provider-todo-panics.md](../auth_sdk/issues/web-provider-todo-panics.md)), and switch cookbook and cahokia web to the component, updating their Playwright tests for the new words. Words and design: [plan](../standards/docs/plans/2026-11-jimhillr-account.md#design).
