# Priority Issues

TODO (agent-generated): [AUTH-001] web - Fix the production OIDC error: `AUTH0_DOMAIN` in `cahokia/web/src/main.rs` must end in `/` to match Auth0's issuer, or OIDC discovery fails on every page load. Copy cookbook's `test_auth0_domain_has_trailing_slash` test. Plan: `cahokia/docs/plans/2026-11-prod-oidc-error.md`, step 1.

TODO (agent-generated): [AUTH-002] web - Use the site origin plus `/` as the OIDC `redirect_url` instead of `window.location.href`, keep the return page in `AppState.return_to`, and clear `code` and `state` from the URL after the token exchange. Plan: `cahokia/docs/plans/2026-11-prod-oidc-error.md`, step 2.

TODO (agent-generated): [AUTH-003] web - Add a Playwright test that the home page never shows "Authentication Error", so a broken sign-in config fails CI. Plan: `cahokia/docs/plans/2026-11-prod-oidc-error.md`, step 4.
