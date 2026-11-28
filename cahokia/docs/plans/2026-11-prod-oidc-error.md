# Plan: fix the OIDC error on Cahokia production

Status: steps 1 and 2 and the Playwright check from step 4 done, 2026-11-26. Reported by Jim: <https://cahokia-web.pages.dev>
shows an OIDC error.

## What is wrong

### 1. The issuer URL is missing its trailing slash (the error you see)

`cahokia/web/src/main.rs` sets `AUTH0_DOMAIN` to
`https://dev-jdadpn4pckxevrv5.us.auth0.com` and passes it to `auth_sdk` as the
OIDC `issuer_url`. Auth0 publishes its issuer with a trailing slash
(`https://dev-jdadpn4pckxevrv5.us.auth0.com/`), and `openidconnect` 4.0.1
compares the two as exact strings (`src/discovery/mod.rs`, line 383). So
discovery fails on every page load, before anyone clicks Login, and the navbar
shows:

> Authentication Error: OIDC discovery failed: Validation error: unexpected
> issuer URI `https://dev-jdadpn4pckxevrv5.us.auth0.com/` (expected
> `https://dev-jdadpn4pckxevrv5.us.auth0.com`)

This is not specific to production: it fails the same way under `dx serve`.
It only became visible now because `cahokia-web` was first deployed by the
mise CD. Cookbook hit the same bug and already fixed it: its `AUTH0_DOMAIN`
ends in `/` and `test_auth0_domain_has_trailing_slash` guards it.

How sure we are: high, from the code and cookbook's fix. The agent sandbox
can't reach Auth0 or the site's runtime, so the exact error text above is the
expected one, not a captured one. Step 1 below checks it.

### 2. The redirect URI is the whole current URL (the error you'll see next)

`App` builds `redirect_url` from `window.location.href`. That breaks two ways:

- Auth0 only accepts a redirect URI that exactly matches an Allowed Callback
  URL. Logging in from `/history` or any page with a query string sends a URI
  that isn't on the list, and Auth0 answers "Callback URL mismatch".
- After Auth0 redirects back, the page reloads with `?code=...&state=...` in
  its URL, so the token exchange sends a different `redirect_uri` than the
  authorize request did. Auth0 rejects that with `invalid_grant`.

### 3. Auth0 doesn't know about the production domain yet (needs Jim)

(Update 2026-11-26: the Pages project was renamed to `jimhillr-cahokia`, so
production is now <https://jimhillr-cahokia.pages.dev>; see "Cloudflare
Pages projects" in standards/docs/naming.md. `issues/human/` has the new
URLs. The steps below keep the old name as it was when written.)

`cahokia-web.pages.dev` is new, so the Auth0 application almost certainly
doesn't list it. The browser calls the token endpoint directly, so Auth0 also
needs the origin for CORS. An agent can't check or change the Auth0
dashboard.

### Ruled out

- **Missing env vars or secrets.** None are needed at runtime. The client ID
  and Auth0 URLs are compiled into the WASM (a public PKCE client has no
  secret). The deploy workflow only needs `CLOUDFLARE_API_TOKEN` and
  `CLOUDFLARE_ACCOUNT_ID`, and the deploy worked: the site answers 200 with
  the title "Cahokia".
- **CSP.** `connect-src` already allows the Auth0 domain for discovery, JWKS
  and the token call.

## Fix

1. **Trailing slash (agent).** Set `AUTH0_DOMAIN` to end in `/`, build
   `AUTH_URL` and `TOKEN_URL` without a double slash, and copy cookbook's
   trailing-slash test. Check with `dx serve` that the navbar shows Login
   instead of the error.
2. **Stable redirect URI (agent).** Use the site origin plus `/` as
   `redirect_url` (`window.location.origin`), keep sending the user back to
   the page they were on through `AppState.return_to`, and clear `code` and
   `state` from the URL with `history.replaceState` once the exchange is done.
3. **Auth0 settings (Jim).** In the Auth0 application Cahokia uses (client ID
   `6CHDECRfCsyYdCFq1hwqKNwCHxxmum3E`), add `https://cahokia-web.pages.dev/`
   and `http://localhost:8080/` to Allowed Callback URLs and
   `https://cahokia-web.pages.dev` and `http://localhost:8080` to Allowed Web
   Origins and Allowed Logout URLs, and check the application type is Single
   Page Application (token endpoint authentication: None). Tracked in
   `issues/human/`.
4. **Stop it coming back (agent).** Give `auth_sdk` one Auth0 helper that
   builds the issuer, authorize and token URLs from a domain, and use it in
   cahokia and cookbook, so the slash can't drift again. Add a Playwright
   check that the home page never renders "Authentication Error".

Steps 1 and 2 can ship in one PR and don't wait on step 3; sign-in works end
to end once all three are done. They touch `cahokia/web/src/main.rs`, which
the family tree work is also changing, so whoever picks them up should rebase
on that work first.

The TODOs are in `cahokia/issues/` and `auth_sdk/docs/issues/`; the Auth0 step
is in `issues/human/`.
