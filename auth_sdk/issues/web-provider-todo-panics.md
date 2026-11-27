---
kind: todo
priority: backlog
tags: [agent-generated]
added: 2026-11-27
---

`WebAuthProvider` implements `is_authenticated`, `is_loading`, `error` and `logout` of the `AuthProvider` trait as `todo!()`, so any app that calls them panics in the browser. cahokia and cookbook don't call them yet, which is the only reason nothing breaks. Implement them: `is_authenticated` from the stored access token, `error` by keeping the `AuthError` from a failed redirect instead of failing `WebAuthProvider::new`, and `logout` by clearing the session state and sending the browser to the provider's `end_session_endpoint` from discovery (Auth0 publishes `/oidc/logout`), with `post_logout_redirect_uri` set to the site root from `redirect::redirect_url_for_origin`. Put the URL building in `redirect.rs` so it tests natively. Drop `is_loading` from the trait if no caller needs it.
