---
kind: todo
priority: backlog
tags: [agent-generated]
epic: "Management app / Phase 8: public site without sign-in"
order: 80
added: 2026-11-28
---

Once the curators use the management app, remove sign-in from the public site: the Login button and auth section in `web`, the `auth_sdk` dependency, Auth0 from the CSP `connect-src` in `web/src` and `web/public/_headers`, the main app's Auth0 callback URLs (credential change), the Login paragraph in the user guide, and Auth0 from `docs/cost.toml` if the management app has its own usage row. Close PR #783 as superseded. Ask Jim before starting, since he said "probably". See the [roadmap](../roadmap.md).
