---
kind: fix
priority: priority
tags: [agent-generated]
added: 2026-11-26
---

The RapidAPI quota guard only learns the quota from response headers. When a response has no `x-ratelimit-requests-remaining` header, `ZillowClient::record_quota` (`../src/main.rs`) saves nothing, so the saved `remaining` never goes down and the guard keeps allowing requests on a stale count. Make the guard conservative: when a request was sent and the response reports no quota, load the saved state and save it again with `remaining` lowered by one (never below zero, keeping `limit` and `resets_at`). Test it with a mockito response that has no quota headers after a saved state of `remaining: 1`, and check that the next request stops without contacting the server. Update "Free quota" in `../README.md`.
