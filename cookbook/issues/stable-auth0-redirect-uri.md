---
kind: fix
priority: backlog
tags: [agent-generated]
added: 2026-11-27
---

`cookbook/web/src/main.rs` sends the whole current page URL as the OAuth `redirect_url` (`fetch_current_location_from_browser`). Auth0 only accepts an exact Allowed Callback URL, so signing in from a recipe or day page, or with a query string, fails with "Callback URL mismatch", and the token exchange after the redirect sends a different `redirect_uri` (now with `code` and `state`) and gets `invalid_grant`. Do what cahokia did in step 2 of `cahokia/docs/plans/2026-11-prod-oidc-error.md`: use `auth_sdk::redirect::redirect_url_for_origin` with `fetch_origin_from_browser`, and check the cookbook's Allowed Callback URLs in Auth0 list the site root.
