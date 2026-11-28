# Plan: native sign-in in auth_sdk

Status: phases 1 and 2 done, 2026-11-28. Asked for so cahokia's desktop and mobile
Login buttons (the `TODO`s in `cahokia/desktop/src/main.rs` and
`cahokia/mobile/src/main.rs`) can sign in with the same Auth0 tenant as the
web app.

## What native sign-in needs

A native app is a public client: it can't keep a client secret, and it
can't be redirected to like a web page. [RFC 8252](https://www.rfc-editor.org/rfc/rfc8252)
(OAuth 2.0 for Native Apps) is the standard answer, and Auth0 follows it:

1. The app opens the authorize URL in the **system browser**, never an
   embedded web view (the app could read the password, and the user's
   existing Auth0 session isn't there).
2. The provider redirects to a URI **the app owns**:
   - desktop: a **loopback** listener, `http://127.0.0.1:<port>/callback`
     (RFC 8252 section 7.3; `127.0.0.1`, not `localhost`, section 8.3);
   - mobile: a **custom URL scheme**, such as
     `com.jimhillr.cahokia://callback` (section 7.1), which Android and iOS
     route to the app.
3. **PKCE** binds the code to the app that started the sign-in, because
   another app could register the same scheme or port (section 8.1).
   auth_sdk already sends PKCE, a `state` and a nonce on the web.

So most of the work is shared with the browser flow. Only "open a URL" and
"get the callback URL back" differ per platform.

## Design

- **`flow` (phase 1):** the authorization code flow without any platform
  calls: discovery, building the authorize URL with fresh PKCE, `state` and
  nonce (`PendingLogin`), parsing a callback URL, checking `state`,
  exchanging the code, and validating the ID token. `web` now uses it too,
  so every security check exists once and is tested natively. The web
  module's tests were wasm32-only and never ran; this moves the logic where
  `cargo test` runs it.
- **`native::NativeAuthClient` (phase 1):** `new(config)` discovers the
  provider, `begin_login()` returns the authorize URL and a `PendingLogin`,
  and `complete_login(pending, callback_url)` finishes. It doesn't open a
  browser or listen on a port, so the same API serves desktop and mobile.
  It doesn't implement the `AuthProvider` trait: that trait's sync
  `login()` with no return value is shaped for a browser redirect, and
  bending it would hide the two steps a native app has to drive.
- **Stricter than the web flow was:** the callback must be to the
  configured redirect URI (scheme, host, port and path), an `error`
  response becomes `AuthError::AuthorizationDenied`, and a token response
  without an ID token is an error (this was the `require-id-token` FIX,
  now done for web and native alike).
- **Loopback ports (phase 2):** Auth0 doesn't accept a wildcard port in
  Allowed Callback URLs (see the Auth0 community threads
  [Are port wildcards supported?](https://community.auth0.com/t/are-port-wildcards-supported-pkce-for-desktop-apps/34132)
  and [Allow wildcard port as per RFC 8252](https://community.auth0.com/t/allow-wildcard-port-in-redirect-uri-as-per-rfc-8252/98409)).
  So the listener tries a short, fixed list of ports, all registered on the
  Auth0 application, and uses the first free one: `DEFAULT_PORTS`, 53682 to
  53685 (phase 2). It binds `127.0.0.1` only, answers one `GET /callback`
  with a "you can close this tab" page and a 404 to anything else, gives up
  after a timeout (five minutes by default), and closes the port as soon as
  the callback arrives. Each connection gets its own task with a short read
  timeout, so a browser's speculative preconnect that never sends can't
  block the real callback. Phase 3 checks whether
  Auth0 has since relaxed this; if it has, the list shrinks to port 0.
- **Tokens stay in memory** until phase 6. Staying signed in across
  restarts needs a refresh token in the OS keychain, which is its own
  decision (scopes, storage, revocation).

## Phases

Each phase is an issue file with the `Native sign-in` epic.

| Phase | What | Where | Issue |
| --- | --- | --- | --- |
| 1 | Shared `flow`, `NativeAuthClient`, tests with a fake provider | auth_sdk | done in this PR |
| 2 | Loopback listener and system browser for desktop | auth_sdk | done: `native::loopback` |
| 3 | A native Auth0 application for cahokia, through IaC | infra | [native-sign-in-phase-3-auth0-native-app.md](../../../infra/docs/issues/native-sign-in-phase-3-auth0-native-app.md) |
| 4 | cahokia desktop Login button signs in | cahokia/desktop | [native-sign-in-phase-4-desktop-login.md](../../../cahokia/desktop/docs/issues/native-sign-in-phase-4-desktop-login.md) |
| 5 | cahokia mobile Login with a custom URL scheme | cahokia/mobile | [native-sign-in-phase-5-mobile-login.md](../../../cahokia/mobile/docs/issues/native-sign-in-phase-5-mobile-login.md) |
| 6 | Stay signed in: refresh token in the OS keychain | auth_sdk | [native-sign-in-phase-6-stay-signed-in.md](../issues/native-sign-in-phase-6-stay-signed-in.md) |

Phase 3 is a credential change (a new Auth0 application and its grants), so
it applies only after Jim approves its run in the `infra-credentials`
environment ([decision 0010](../../../standards/docs/decisions/0010-credential-second-approval.md)).
The application's client ID is public, like the web app's, so no secret is
made or stored. Phase 4 can be built and tested against the fake provider
before phase 3 lands, but can't be checked end to end until it does.

## Cost

$0. Auth0's free plan covers native applications and the same monthly
active users; one more application doesn't change the tier. No new vendor.
Phase 2 adds the `webbrowser` crate (open source) to open the system
browser. Claude Code effort: phases 2, 4 and 6 are each about one thread;
phases 3 and 5 are smaller in code but wait on Jim (approval) and on a
spike into Dioxus mobile deep links.

## Risks

- **Another local app grabs the port first** and receives the callback.
  PKCE means it can't exchange the code, and `state` must match, so the
  worst case is a failed sign-in (RFC 8252 section 8.3). The listener binds
  `127.0.0.1` only, never all interfaces.
- **Custom scheme hijacking on mobile**: same PKCE protection. Claimed
  `https` app links are stronger and a later option.
- **Embedded web views**: the API only returns a URL; the docs say to open
  it in the system browser.
